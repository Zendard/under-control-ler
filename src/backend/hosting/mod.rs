#[cfg(target_os = "linux")]
use self::linux::VirtualGamepad;
use super::{open_socket, NetworkMessageSender};
use crate::backend::NetworkMessage;
use crate::{BackendMessage, FrontendMessage};
use iced::futures::channel::mpsc;
use iced::futures::executor::block_on;
use iced::futures::SinkExt;
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
    let mut accepted_clients: Vec<Client> = vec![];
    // Obtain a UDP socket
    let socket = open_socket(port);
    // Initialize a ThreadPool for handling requests
    let pool = threadpool::ThreadPool::new(10);
    println!("Hosting...");

    loop {
        // Check for new ui messages
        let ui_message = receiver.try_next();

        // Stop when receiving StopHosting message
        match ui_message {
            Ok(Some(FrontendMessage::StopHosting)) => break,
            Ok(Some(FrontendMessage::AcceptClient(origin))) => {
                accept_client(&mut accepted_clients, origin)
            }
            _ => (),
        }

        // Check for new network requests
        let received_data = socket.next_message();

        // Skip request handling when message is an error
        if received_data.is_none() {
            dbg!(&received_data);
            continue;
        }

        let (message, origin) = received_data.unwrap();
        dbg!(&message);

        // Origin doesn't need to be accepted to ping, so we run it before checking
        if let NetworkMessage::Ping = message {
            let socket_clone = socket.try_clone().unwrap();
            pool.execute(move || {
                let sender = NetworkMessageSender {
                    socket: socket_clone,
                    destination: origin,
                };
                sender.send_network_message(NetworkMessage::Ping).unwrap();
            });
            // We don't want to send a join request when just pinging
            continue;
        }

        // Check if origin is already accepted
        let accepted = accepted_clients
            .iter()
            .map(|client| client.address)
            .any(|address| address == origin);

        // If the origin is not accepted, send a JoinRequest to frontend and skip further handling
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

fn accept_client(accepted_clients: &mut Vec<Client>, address: SocketAddr) {
    let gamepad = VirtualGamepad::new().expect("Failed to create virtual gamepad");
    let client = Client {
        gamepad: Arc::new(Mutex::new(gamepad)),
        address,
    };

    accepted_clients.push(client);
}
