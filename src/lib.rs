use std::net::UdpSocket;

#[derive(Debug)]
pub struct JoinConfig {
    pub address: String,
    pub port: String,
}

#[derive(Debug)]
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

pub fn host(host_config: HostConfig) {
    let port = host_config.port;

    let socket = UdpSocket::bind(format!("0.0.0.0:{port}"))
        .expect(&format!("Failed to bind to port {port}"));

    let mut receive_buffer = [];
    while let Ok((n, addr)) = socket.recv_from(&mut receive_buffer) {
        println!("{} bytes response from {:?}", n, addr);
        // Remaining code not directly relevant to the question
    }
}
