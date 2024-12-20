use crate::{BackendMessage, FrontendMessage};
use evdev::uinput::VirtualDevice;
use iced::futures::channel::mpsc;
use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};

mod linux;
mod threadpool;

const DEFAULT_PORT: u16 = 8629;

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

pub fn host(sender: mpsc::Sender<BackendMessage>, receiver: mpsc::Receiver<FrontendMessage>) {
    #[cfg(target_os = "linux")]
    std::thread::spawn(move || async move {
        linux::host(DEFAULT_PORT, sender, receiver).await;
    });
    #[cfg(target_os = "windows")]
    windows::host(DEFAULT_PORT, sender, reciever);
}
