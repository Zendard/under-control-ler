use super::{open_socket, NetworkMessage, NetworkMessageSender};
use crate::{AxisInput, BackendMessage, ButtonInput, FrontendMessage, GamepadInput};
use gilrs::{Axis, Button, Event, Gilrs};
use iced::futures::{
    channel::mpsc::{Receiver, Sender},
    executor::block_on,
    SinkExt, StreamExt,
};
use std::{net::SocketAddr, time::Duration};

pub fn join(
    socket_addr: SocketAddr,
    mut sender: Sender<BackendMessage>,
    mut receiver: Receiver<FrontendMessage>,
) {
    let network_sender = NetworkMessageSender {
        // Add 1 to port so you can join and host on the same machine for testing
        socket: open_socket(socket_addr.port() + 1),
        destination: socket_addr,
    };
    println!("Joining {}", socket_addr);
    ping(
        network_sender.try_clone().unwrap(),
        socket_addr,
        sender.clone(),
    );
    network_sender
        .send_network_message(NetworkMessage::JoinRequest)
        .unwrap();

    // Check for new ui messages
    std::thread::spawn(move || loop {
        let ui_message = block_on(receiver.select_next_some());
        dbg!(&ui_message);
    });

    // Wait until we are accepted
    loop {
        // Check for new network requests
        let received_data = network_sender.socket.next_message();
        dbg!(&received_data);

        // Only continue when received data is not an error and a ClientAccepted message
        if let Some((NetworkMessage::ClientAccepted, origin)) = received_data {
            dbg!(&received_data);
            // Only break when origin is the host we want to connect to
            if origin == network_sender.destination {
                break;
            }
        }
    }
    // Send accepted message to frontend
    block_on(sender.send(BackendMessage::Client(crate::UIMessageClient::Accepted))).unwrap();
    println!("We were accepted");

    let mut gilrs = Gilrs::new().unwrap();
    let mut now = std::time::Instant::now();

    // Listen for inputs
    loop {
        while let Some(Event { id, event, .. }) = gilrs.next_event() {
            // Skip handling when vendor id is our own
            if gilrs.gamepad(id).vendor_id() == Some(8629) {
                continue;
            }

            dbg!(&event);
            // Only send input when we can convert it to a GamepadInput
            if let Some(event) = GamepadInput::from_event(event) {
                dbg!(&event);
                network_sender
                    .send_network_message(NetworkMessage::Input(event))
                    .unwrap();
            }
            if now.elapsed() >= std::time::Duration::new(1, 0) {
                ping(
                    network_sender.try_clone().unwrap(),
                    socket_addr,
                    sender.clone(),
                );
                now = std::time::Instant::now();
            }
        }
    }
}

fn ping(socket: NetworkMessageSender, socket_addr: SocketAddr, mut sender: Sender<BackendMessage>) {
    socket
        .send_network_message(NetworkMessage::Ping)
        .expect("Failed to send message");
    let now = std::time::Instant::now();
    // Some random NetworkMessage variant which isn't Ping
    let mut message = NetworkMessage::ClientAccepted;
    // Keep waiting for Ping with a 5 second timeout
    while message != NetworkMessage::Ping && now.elapsed() < Duration::from_secs(5) {
        let received_data = socket.socket.next_message();

        if received_data.is_none() {
            continue;
        }
        let (received_message, received_origin) = received_data.unwrap();

        // Only accept ping when from the same origin
        if received_origin == socket_addr {
            message = received_message
        }
    }
    let ping_ms = now.elapsed().as_nanos() as f32 / 1_000_000_f32;
    block_on(
        sender.send(BackendMessage::Client(crate::UIMessageClient::Ping(
            ping_ms,
        ))),
    )
    .unwrap();
}

impl GamepadInput {
    fn from_event(event: gilrs::EventType) -> Option<Self> {
        match event {
            gilrs::EventType::ButtonPressed(button, _) => Self::convert_button(button, 1.0),
            gilrs::EventType::ButtonReleased(button, _) => Self::convert_button(button, 0.0),
            gilrs::EventType::AxisChanged(axis, value, _) => Self::convert_axis(axis, value),
            _ => None,
        }
    }

    fn convert_axis(axis: Axis, value: f32) -> Option<Self> {
        let axis = match axis {
            Axis::LeftStickX => AxisInput::StickLeftX,
            Axis::LeftStickY => AxisInput::StickLeftY,
            Axis::RightStickX => AxisInput::StickRightX,
            Axis::RightStickY => AxisInput::StickRightY,
            Axis::LeftZ => AxisInput::TriggerLeft,
            Axis::RightZ => AxisInput::TriggerRight,
            _ => return None,
        };
        // Axis values range from -1 to 1, so we multiply by 127 to maximise the i8 range
        let value = value * 127.0;
        Some(GamepadInput::Axis(axis, value.round() as i8))
    }

    fn convert_button(button: Button, value: f32) -> Option<Self> {
        // Triggers are treated as buttons by gilrs, we treat them as axis
        if let Button::LeftTrigger2 = button {
            let value = value * 127.0;
            return Some(GamepadInput::Axis(
                AxisInput::TriggerLeft,
                value.round() as i8,
            ));
        };
        // Same with right trigger
        if let Button::RightTrigger2 = button {
            let value = value * 127.0;
            return Some(GamepadInput::Axis(
                AxisInput::TriggerRight,
                value.round() as i8,
            ));
        };
        let button = match button {
            Button::South => ButtonInput::A,
            Button::East => ButtonInput::B,
            Button::West => ButtonInput::X,
            Button::North => ButtonInput::Y,
            Button::DPadUp => ButtonInput::DpadUp,
            Button::DPadDown => ButtonInput::DpadDown,
            Button::DPadLeft => ButtonInput::DpadLeft,
            Button::DPadRight => ButtonInput::DpadRight,
            Button::LeftTrigger => ButtonInput::BumperLeft,
            Button::RightTrigger => ButtonInput::BumperRight,
            Button::LeftThumb => ButtonInput::StickLeft,
            Button::RightThumb => ButtonInput::StickRight,
            Button::Select => ButtonInput::Select,
            Button::Start => ButtonInput::Start,
            _ => return None,
        };
        Some(GamepadInput::Button(button, value == 1.0))
    }
}
