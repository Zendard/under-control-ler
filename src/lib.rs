use std::{
    net::UdpSocket,
    sync::{Arc, Mutex},
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
    // println!("received {n} from {addr}");
}

fn make_connection(join_config: JoinConfig) {
    let address = join_config.address;
    let port = join_config.port;
    let socket = UdpSocket::bind(format!("0.0.0.0:0")).expect(&format!("Failed to bind UdpSocket"));
    socket.set_broadcast(true).unwrap();

    socket
        .connect(format!("{address}:{port}"))
        .expect("Failed to connect to {address} on port {port}");

    socket.send(b"Test").unwrap();
}

pub fn host(host_config: HostConfig, messages: Arc<Mutex<Vec<String>>>) {
    let port = host_config.port;

    let socket = UdpSocket::bind(format!("0.0.0.0:{port}"))
        .expect(&format!("Failed to bind to port {port}"));

    thread::spawn(move || {
        let mut buffer = [];
        while let Ok(_) = socket.recv_from(&mut buffer) {
            messages
                .lock()
                .unwrap()
                .push(String::from_utf8(buffer.to_vec()).unwrap_or("Not valid utf-8".to_string()));
        }
    });
}
