use super::open_socket;
use crate::{BackendMessage, FrontendMessage};
#[cfg(target_os = "linux")]
use evdev::uinput::VirtualDevice;
use iced::futures::executor::{block_on, ThreadPool};
use iced::futures::SinkExt;
use iced::futures::{channel::mpsc, StreamExt};
use std::net::{Ipv4Addr, SocketAddrV4, UdpSocket};
use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};

mod linux;

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
    mut sender: mpsc::Sender<BackendMessage>, // Sender for ui events
    mut receiver: mpsc::Receiver<FrontendMessage>, // Receiver for ui events
) {
    // Start with no clients accepted
    let accepted_clients: Vec<Client> = vec![];
    // Obtain a UDP socket
    let socket = open_socket(port);
    // Initialize a ThreadPool for handling requests
    let pool = ThreadPool::new().unwrap();
    println!("Hosting...");

    loop {
        // Check for new ui messages
        let ui_message = receiver.try_next();

        // Stop when receiving StopHosting message
        if let Ok(Some(FrontendMessage::StopHosting)) = ui_message {
            break;
        }

        // Check for new network requests
        let received_data = socket.next_message();

        // Skip request handling when message is an error
        if received_data.is_none() {
            dbg!(&received_data);
            continue;
        }

        let (message, origin) = received_data.unwrap();
        dbg!(message);

        // Check if origin is already accepted
        let accepted = accepted_clients
            .iter()
            .map(|client| client.address)
            .any(|address| address == origin);

        // If the origin is not accepted, send a JoinRequest to frontend
        if !accepted {
            block_on(
                sender.send(BackendMessage::Server(crate::UIMessageServer::JoinRequest(
                    origin,
                ))),
            )
            .unwrap();
            continue;
        }
    }
    println!("Stopped hosting")
}
