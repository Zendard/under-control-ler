use std::{
    net::UdpSocket,
    sync::mpsc::{self, Receiver},
    thread,
};

pub struct JoinConfig {
    pub address: String,
    pub port: String,
}

pub struct HostConfig {
    pub port: String,
}

pub fn join(join_config: JoinConfig) {
    make_connection(join_config)
}

fn make_connection(join_config: JoinConfig) {
    let address = join_config.address;
    let port = join_config.port;
    let socket = UdpSocket::bind(format!("127.0.0.1:{port}"))
        .expect(&format!("Failed to bind to port {port}"));

    socket
        .connect(format!("{address}:{port}"))
        .expect("Failed to connect to {address} on port {port}");

    socket.send(b"Test").unwrap();
}

pub fn host(host_config: HostConfig) -> Receiver<String> {
    let port = host_config.port;

    let socket = UdpSocket::bind(format!("0.0.0.0:{port}"))
        .expect(&format!("Failed to bind to port {port}"));

    let mut receive_buffer = [];
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        while let Ok((n, addr)) = socket.recv_from(&mut receive_buffer) {
            tx.send(n.to_string()).unwrap();
            println!("{} bytes response from {:?}", n, addr);
            // Remaining code not directly relevant to the question
        }
    });

    rx
}
