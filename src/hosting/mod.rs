use evdev::uinput::VirtualDevice;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

pub mod linux;

#[derive(Debug)]
struct RawMessage {
    data: [u8; 100],
    length: usize,
    origin: SocketAddr,
}

#[cfg(target_os = "linux")]
pub struct VirtualGamepad(VirtualDevice);

pub struct Client {
    pub gamepad: Arc<Mutex<VirtualGamepad>>,
    pub address: SocketAddr,
}
