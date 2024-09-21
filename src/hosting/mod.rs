use std::net::SocketAddr;

pub mod linux;
pub mod windows;

#[derive(Debug)]
struct RawMessage {
    data: [u8; 100],
    length: usize,
    origin: SocketAddr,
}
