use crate::{BackendMessage, FrontendMessage};
#[cfg(target_os = "linux")]
use evdev::uinput::VirtualDevice;
use iced::futures::executor::block_on;
use iced::futures::{channel::mpsc, StreamExt};
use std::net::{Ipv4Addr, SocketAddrV4, UdpSocket};
use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};

mod linux;
mod threadpool;

#[derive(Debug)]
struct RawMessage {
    data: [u8; 100],
    length: usize,
    origin: SocketAddr,
}

#[cfg(target_os = "linux")]
pub struct VirtualGamepad(VirtualDevice);

#[cfg(target_os = "windows")]
pub struct VirtualGamepad();

struct Client {
    pub gamepad: Arc<Mutex<VirtualGamepad>>,
    pub address: SocketAddr,
}

pub fn host(
    port: u16,
    sender: mpsc::Sender<BackendMessage>,
    mut receiver: mpsc::Receiver<FrontendMessage>,
) {
    let accepted_clients: Vec<Client> = vec![];

    let socket = open_socket(port);

    println!("Hosting...");

    loop {
        let message = block_on(receiver.select_next_some());

        dbg!(&message);

        if message == FrontendMessage::StopHosting {
            dbg!("Stopping");
            break;
        }
    }
    println!("Stopped hosting")
}

fn open_socket(port: u16) -> UdpSocket {
    UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), port))
        .expect("Failed to bind to port")
}
