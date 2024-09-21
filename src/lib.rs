use gilrs::{ev::AxisOrBtn, EventType, Gilrs};
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket},
    str::FromStr,
};
#[cfg(target_os = "linux")]
mod hosting;

pub struct JoinConfig {
    socket: SocketAddr,
}

impl JoinConfig {
    fn new(args: &[String]) -> JoinConfig {
        let ip_address = args.get(2).expect("Please enter an address");
        let ip_address = IpAddr::from_str(ip_address).expect("Please enter a valid ipv4 or ipv6");

        let default_port = "8629".to_string();
        let port = args.get(3).unwrap_or(&default_port);
        let port: u16 = port.parse().expect("Port number must be u16");

        let socket = SocketAddr::new(ip_address, port);

        JoinConfig { socket }
    }
}

pub struct HostConfig {
    pub port: u16,
}

impl HostConfig {
    fn new(args: &[String]) -> HostConfig {
        let default_port = "8629".to_string();
        let port = args.get(2).unwrap_or(&default_port);
        let port: u16 = port.parse().expect("Port number must be u16");

        HostConfig { port }
    }
}

const JOYSTICK_RANGE: isize = 32768;
const TRIGGER_RANGE: isize = 1023;

pub fn join(args: &[String]) {
    let config = JoinConfig::new(args);

    let socket = make_connection(&config);
    send_controller_inputs(socket);
}

#[cfg(target_os = "linux")]
pub fn host(args: &[String]) {
    crate::hosting::linux::host(args);
}

#[cfg(target_os = "windows")]
pub fn host(args: &[String]) {
    crate::hosting::windows::host(args);
}

fn make_connection(join_config: &JoinConfig) -> UdpSocket {
    let socket = UdpSocket::bind(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0))
        .expect("Failed to bind socket");

    socket
        .connect(join_config.socket)
        .unwrap_or_else(|error| panic!("Failed to connect to {}: {}", join_config.socket, error));

    socket.send(b"Joined").unwrap();

    socket
}

fn send_controller_inputs(socket: UdpSocket) {
    let mut girls = Gilrs::new().unwrap();
    let gamepad_names = girls
        .gamepads()
        .map(|gamepad| gamepad.1.name().to_string())
        .collect::<Vec<String>>();

    println!("Detected gamepads: {:?}", gamepad_names);
    println!(
        "Connected to {}, sending inputs...",
        &socket.peer_addr().unwrap()
    );

    loop {
        handle_controller_event(&mut girls, &socket)
    }
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
