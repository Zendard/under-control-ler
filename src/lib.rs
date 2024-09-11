use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket},
    str::FromStr,
    thread,
};

use gilrs::{EventType, Gilrs};

pub struct JoinConfig {
    socket: SocketAddr,
}

impl JoinConfig {
    fn new(args: &Vec<String>) -> JoinConfig {
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
    fn new(args: &Vec<String>) -> HostConfig {
        let default_port = "8629".to_string();
        let port = args.get(2).unwrap_or(&default_port);
        let port: u16 = port.parse().expect("Port number must be u16");

        HostConfig { port }
    }
}

#[derive(Debug)]
struct RawMessage {
    data: [u8; 68],
    length: usize,
    origin: SocketAddr,
}

pub fn join(args: &Vec<String>) {
    let config = JoinConfig::new(args);

    let socket = make_connection(&config);
    send_controller_inputs(socket);
}

pub fn host(args: &Vec<String>) {
    let config = HostConfig::new(args);

    open_port(&config);
}

fn make_connection(join_config: &JoinConfig) -> UdpSocket {
    let socket = UdpSocket::bind(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0))
        .expect("Failed to bind socket");

    socket
        .connect(join_config.socket)
        .expect("Failed to connect to {address} on port {port}");

    socket.send(b"Joined").unwrap();

    socket
}

fn send_controller_inputs(socket: UdpSocket) {
    let mut girls = Gilrs::new().unwrap();

    loop {
        handle_controller_event(&mut girls, &socket)
    }
}

fn handle_controller_event(girls: &mut Gilrs, socket: &UdpSocket) {
    while let Some(event) = girls.next_event() {
        let event = event.event;
        send_controller_event(event, socket);
    }
}

fn send_controller_event(event: EventType, socket: &UdpSocket) {
    let event_string = serde_json::to_string(&event).unwrap();
    let buffer: &[u8] = event_string.as_bytes();
    socket.send(buffer).unwrap_or_else(|error| {
        eprintln!("{}", error);
        0
    });
}

fn open_port(config: &HostConfig) {
    let socket = UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), config.port))
        .expect("Failed to bind to port");

    let mut receive_buffer = [0; 68];
    while let Ok((length, origin)) = socket.recv_from(&mut receive_buffer) {
        let data = receive_buffer.clone();
        let message = RawMessage {
            data,
            length,
            origin,
        };
        thread::spawn(move || handle_receive(message));
    }
}

fn handle_receive(message: RawMessage) {
    let data = &message.data[..message.length];
    let message_string = String::from_utf8(data.to_vec()).unwrap_or("Not valid utf-8".to_string());
    let event: Option<EventType> = serde_json::from_str(&message_string).unwrap_or(None);
    dbg!(event);
}
