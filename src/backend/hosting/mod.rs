#[cfg(target_os = "linux")]
use self::linux::VirtualGamepad;
#[cfg(target_os = "windows")]
use self::windows::VirtualGamepad;
use super::{open_socket, NetworkMessageSender, NetworkMessageSocket};
use crate::backend::NetworkMessage;
use crate::{BackendMessage, FrontendMessage};
use iced::futures::channel::mpsc;
use iced::futures::executor::block_on;
use iced::futures::{SinkExt, StreamExt};
use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};

#[cfg(target_os = "linux")]
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
    let accepted_clients: Arc<Mutex<Vec<Client>>> = Arc::new(Mutex::new(Vec::new()));
    // Obtain a UDP socket
    let socket = open_socket(port);
    // Initialize a ThreadPool for handling requests
    let pool = threadpool::ThreadPool::new(10);
    println!("Hosting...");

    let hosting = Arc::new(Mutex::new(true));

    // We have to clone the values used in the network thread to satisfy the compiler
    let socket_clone = socket.try_clone().unwrap();
    let accepted_clients_clone = accepted_clients.clone();
    let hosting_clone = hosting.clone();

    // Check for new network requests
    std::thread::spawn(move || {
        while *hosting_clone.lock().unwrap() {
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
                .lock()
                .unwrap()
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

            // Play input when receiving input, after acceptation check
            if let NetworkMessage::Input(input) = message {
                accepted_clients
                    .lock()
                    .unwrap()
                    .iter()
                    .find(|client| client.address == origin)
                    .unwrap()
                    .gamepad
                    .lock()
                    .unwrap()
                    .play_input(input);
            }
        }
    });

    // Check for new ui messages
    loop {
        let ui_message = block_on(receiver.select_next_some());
        dbg!(&ui_message);

        // Stop when receiving StopHosting message
        match ui_message {
            FrontendMessage::StopHosting => {
                *hosting.lock().unwrap() = false;
                break;
            }
            FrontendMessage::AcceptClient(origin) => {
                println!("Accepted {}", origin);
                accept_client(&accepted_clients_clone, origin, &socket_clone);
            }
            _ => (),
        }
    }
    println!("Stopped hosting");
}

fn accept_client(
    accepted_clients: &Arc<Mutex<Vec<Client>>>,
    address: SocketAddr,
    socket: &NetworkMessageSocket,
) {
    // Create virtual gamepad
    let gamepad = VirtualGamepad::new().expect("Failed to create virtual gamepad");
    let client = Client {
        gamepad: Arc::new(Mutex::new(gamepad)),
        address,
    };

    // Add client to list of accepted clients
    accepted_clients.lock().unwrap().push(client);

    // Let client know they were accepted
    let sender = NetworkMessageSender {
        socket: socket.try_clone().unwrap(),
        destination: address,
    };
    sender
        .send_network_message(NetworkMessage::ClientAccepted)
        .unwrap();
}
