use gilrs::{ev::AxisOrBtn, EventType, Gilrs};
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket},
    sync::{
        mpsc::{Receiver, Sender},
        Arc, Mutex,
    },
};
#[cfg(target_os = "linux")]
mod hosting;

const JOYSTICK_RANGE: isize = 32768;
const TRIGGER_RANGE: isize = 1023;

#[derive(Clone, Copy, Debug)]
pub enum Message {
    ClientJoined(SocketAddr),
    ClientAccepted(SocketAddr),
    ClientRejected(SocketAddr),
}

pub fn join(address: SocketAddr, stop: Arc<Mutex<bool>>) {
    let socket = make_connection(&address);
    send_controller_inputs(socket, stop);
}

#[cfg(target_os = "linux")]
pub fn host(
    port: u16,
    stop: Arc<Mutex<bool>>,
    sender: Sender<crate::Message>,
    receiver: Receiver<crate::Message>,
) {
    crate::hosting::linux::host(port, stop, sender, receiver);
}

#[cfg(target_os = "windows")]
pub fn host(
    port: u16,
    stop: Arc<Mutex<bool>>,
    sender: Sender<crate::Message>,
    receiver: Receiver<crate::Message>,
) {
    eprintln!("Hosting only supported on linux for now...")
}
fn make_connection(address: &SocketAddr) -> UdpSocket {
    let socket = UdpSocket::bind(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0))
        .expect("Failed to bind socket");

    socket
        .connect(address)
        .unwrap_or_else(|error| panic!("Failed to connect to {}: {}", address, error));

    socket.send(b"Joined").unwrap();

    socket
}

fn send_controller_inputs(socket: UdpSocket, stop: Arc<Mutex<bool>>) {
    let mut gilrs = Gilrs::new().unwrap();
    let gamepad_names = gilrs
        .gamepads()
        .map(|gamepad| gamepad.1.name().to_string())
        .collect::<Vec<String>>();

    println!("Detected gamepads: {:?}", gamepad_names);
    println!(
        "Connected to {}, sending inputs...",
        &socket.peer_addr().unwrap()
    );

    while !*stop.lock().unwrap() {
        handle_controller_event(&mut gilrs, &socket)
    }
    println!("Left");
}

fn handle_controller_event(girls: &mut Gilrs, socket: &UdpSocket) {
    while let Some(event) = girls.next_event() {
        if girls.gamepad(event.id).vendor_id() == Some(8629) {
            return;
        }

        let event = event.event;

        send_controller_event(event, socket);
    }
}

fn send_controller_event(event: EventType, socket: &UdpSocket) {
    let event_string = SimpleEventType(event).to_universal_string();
    let buffer: &[u8] = event_string.as_bytes();
    socket.send(buffer).unwrap_or_else(|error| {
        eprintln!("{}", error);
        0
    });
}

#[derive(Debug)]
struct SimpleEventType(EventType);

impl SimpleEventType {
    fn to_universal_string(&self) -> String {
        match self.0 {
            EventType::ButtonChanged(button, value, _) => {
                format!("b,{},{}", serde_json::to_string(&button).unwrap(), value)
            }
            EventType::AxisChanged(axis, value, _) => {
                format!(
                    "a,{:?},{}",
                    serde_json::to_string(&axis)
                        .unwrap()
                        .replace(['\"', '\\'], ""),
                    value
                )
            }
            _ => "".to_string(),
        }
    }

    fn from_string(string: &str) -> Option<(AxisOrBtn, f32)> {
        let mut parts = string.split(',');
        let variant = parts.next()?;
        let name = parts.next()?;
        let value = parts.next()?.parse::<f32>().ok()?;

        match variant {
            "b" => Some((AxisOrBtn::Btn(serde_json::from_str(name).ok()?), value)),
            "a" => Some((AxisOrBtn::Axis(serde_json::from_str(name).ok()?), value)),
            _ => None,
        }
    }
}
