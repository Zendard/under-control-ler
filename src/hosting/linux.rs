use crate::{hosting::RawMessage, HostConfig, SimpleEventType, JOYSTICK_RANGE, TRIGGER_RANGE};
use evdev::{
    uinput::{VirtualDevice, VirtualDeviceBuilder},
    AbsInfo, AbsoluteAxisType, AttributeSet, BusType, InputEvent, InputId, Key, UinputAbsSetup,
};
use std::{
    error::Error,
    net::{Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket},
    rc::Rc,
    sync::{Arc, Mutex},
    thread,
};

use gilrs::{ev::AxisOrBtn, Axis, Button};

struct VirtualGamepad(VirtualDevice);

impl VirtualGamepad {
    fn new() -> Result<Self, Box<dyn Error>> {
        let mut keys = AttributeSet::new();
        keys.insert(Key::BTN_NORTH);
        keys.insert(Key::BTN_EAST);
        keys.insert(Key::BTN_WEST);
        keys.insert(Key::BTN_SOUTH);

        keys.insert(Key::BTN_DPAD_UP);
        keys.insert(Key::BTN_DPAD_DOWN);
        keys.insert(Key::BTN_DPAD_LEFT);
        keys.insert(Key::BTN_DPAD_RIGHT);

        keys.insert(Key::BTN_SELECT);
        keys.insert(Key::BTN_START);

        keys.insert(Key::BTN_TL);
        keys.insert(Key::BTN_TR);

        keys.insert(Key::BTN_THUMBL);
        keys.insert(Key::BTN_THUMBR);

        let abs_setup = AbsInfo::new(2293, -32768, 32767, 16, 128, 1);
        let trigger_setup = AbsInfo::new(2293, 0, 1023, 16, 128, 1);
        let left_x = UinputAbsSetup::new(AbsoluteAxisType::ABS_X, abs_setup);
        let left_y = UinputAbsSetup::new(AbsoluteAxisType::ABS_Y, abs_setup);

        let right_x = UinputAbsSetup::new(AbsoluteAxisType::ABS_RX, abs_setup);
        let right_y = UinputAbsSetup::new(AbsoluteAxisType::ABS_RY, abs_setup);

        let left_trigger = UinputAbsSetup::new(AbsoluteAxisType::ABS_Z, trigger_setup);
        let right_trigger = UinputAbsSetup::new(AbsoluteAxisType::ABS_RZ, trigger_setup);

        let gamepad = VirtualDeviceBuilder::new()?
            .name("Under Control(ler) Virtual Gamepad")
            .input_id(InputId::new(BusType::BUS_USB, 8629, 8629, 1))
            .with_keys(&keys)?
            .with_absolute_axis(&left_x)?
            .with_absolute_axis(&left_y)?
            .with_absolute_axis(&right_x)?
            .with_absolute_axis(&right_y)?
            .with_absolute_axis(&left_trigger)?
            .with_absolute_axis(&right_trigger)?
            .build()?;

        Ok(VirtualGamepad(gamepad))
    }

    fn set_key(&mut self, key: Key, value: i32) {
        self.0
            .emit(&[InputEvent::new(evdev::EventType::KEY, key.code(), value)])
            .unwrap()
    }

    fn set_axis(&mut self, axis: AbsoluteAxisType, value: f32) {
        let mut value = (value * JOYSTICK_RANGE as f32) as i32;

        if AbsoluteAxisType::ABS_Y == axis || AbsoluteAxisType::ABS_RY == axis {
            value = -value
        }

        self.0
            .emit(&[InputEvent::new(evdev::EventType::ABSOLUTE, axis.0, value)])
            .unwrap()
    }

    fn set_trigger(&mut self, axis: AbsoluteAxisType, value: f32) {
        let value = (value * TRIGGER_RANGE as f32) as i32;

        self.0
            .emit(&[InputEvent::new(evdev::EventType::ABSOLUTE, axis.0, value)])
            .unwrap()
    }
}

struct Client {
    pub gamepad: Arc<Mutex<VirtualGamepad>>,
    pub address: SocketAddr,
}

pub fn host(args: &[String]) {
    let config = HostConfig::new(args);

    open_port(&config);
}

fn open_port(config: &HostConfig) {
    let socket = UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), config.port))
        .expect("Failed to bind to port");

    let clients: Rc<Mutex<Vec<Client>>> = Rc::new(Mutex::new(Vec::new()));

    let mut receive_buffer = [0; 100];
    println!("Waiting for inputs...");
    while let Ok((length, origin)) = socket.recv_from(&mut receive_buffer) {
        let data = receive_buffer;
        let message = RawMessage {
            data,
            length,
            origin,
        };
        let gamepad = find_or_create_client(message.origin, clients.clone());
        if gamepad.is_none() {
            continue;
        }
        thread::spawn(move || handle_receive(message, gamepad.unwrap()));
    }
}

fn find_or_create_client(
    address: SocketAddr,
    clients: Rc<Mutex<Vec<Client>>>,
) -> Option<Arc<Mutex<VirtualGamepad>>> {
    let mut clients = clients.lock().unwrap();
    let client = clients.iter().find(|client| client.address == address);

    match client {
        None => {
            let confirmation = dialoguer::Confirm::new()
                .with_prompt(format!("Accept connection from {address}?"))
                .interact()
                .unwrap();

            if confirmation {
                let gamepad = Arc::new(Mutex::new(VirtualGamepad::new().ok()?));
                clients.push(Client {
                    gamepad: gamepad.clone(),
                    address,
                });
                Some(gamepad)
            } else {
                None
            }
        }
        Some(client) => Some(client.gamepad.clone()),
    }
}

fn handle_receive(message: RawMessage, gamepad: Arc<Mutex<VirtualGamepad>>) {
    let data = &message.data[..message.length];
    let message_string = String::from_utf8(data.to_vec()).unwrap_or("Not valid utf-8".to_string());
    let event: Option<(AxisOrBtn, f32)> = SimpleEventType::from_string(&message_string);

    let Some(event) = event else { return };

    match event.0 {
        AxisOrBtn::Btn(button) => handle_button_changed(button, event.1, gamepad),
        AxisOrBtn::Axis(axis) => handle_axis_changed(axis, event.1, gamepad),
    }
}

fn handle_button_changed(button: Button, value: f32, gamepad: Arc<Mutex<VirtualGamepad>>) {
    let key = translate_button(button);

    match button {
        Button::LeftTrigger2 => gamepad
            .lock()
            .unwrap()
            .set_trigger(AbsoluteAxisType::ABS_Z, value),
        Button::RightTrigger2 => gamepad
            .lock()
            .unwrap()
            .set_trigger(AbsoluteAxisType::ABS_RZ, value),
        _ => {}
    };
    gamepad.lock().unwrap().set_key(key, value as i32);
}

fn translate_button(button: Button) -> Key {
    match button {
        Button::North => Key::BTN_NORTH,
        Button::East => Key::BTN_EAST,
        Button::West => Key::BTN_WEST,
        Button::South => Key::BTN_SOUTH,

        Button::DPadUp => Key::BTN_DPAD_UP,
        Button::DPadDown => Key::BTN_DPAD_DOWN,
        Button::DPadLeft => Key::BTN_DPAD_LEFT,
        Button::DPadRight => Key::BTN_DPAD_RIGHT,

        Button::Select => Key::BTN_SELECT,
        Button::Start => Key::BTN_START,

        Button::LeftTrigger => Key::BTN_TL,
        Button::RightTrigger => Key::BTN_TR,

        Button::LeftThumb => Key::BTN_THUMBL,
        Button::RightThumb => Key::BTN_THUMBR,

        _ => Key::BTN_NORTH,
    }
}

fn handle_axis_changed(axis: Axis, value: f32, gamepad: Arc<Mutex<VirtualGamepad>>) {
    let axis = translate_axis(axis);
    gamepad.lock().unwrap().set_axis(axis, value);
}

fn translate_axis(axis: Axis) -> AbsoluteAxisType {
    match axis {
        Axis::LeftStickX => AbsoluteAxisType::ABS_X,
        Axis::LeftStickY => AbsoluteAxisType::ABS_Y,

        Axis::RightStickX => AbsoluteAxisType::ABS_RX,
        Axis::RightStickY => AbsoluteAxisType::ABS_RY,

        _ => AbsoluteAxisType::ABS_X,
    }
}
