use std::net::UdpSocket;

#[derive(Debug, Default)]
pub struct JoinConfig {
    pub address: String,
    pub port: String,
}

pub fn join(join_config: JoinConfig) {
    make_connection(join_config)
}

fn make_connection(join_config: JoinConfig) {
    let address = join_config.address;
    let port = join_config.port;
    UdpSocket::bind(format!("127.0.0.1:{port}")).expect("Something is already using this port!");
}
