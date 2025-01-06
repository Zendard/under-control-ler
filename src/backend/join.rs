use super::{open_socket, NetworkMessage, NetworkMessageSender};
use crate::{BackendMessage, FrontendMessage};
use iced::futures::{
    channel::mpsc::{Receiver, Sender},
    executor::block_on,
    SinkExt,
};
use std::{net::SocketAddr, time::Duration};

pub fn join(
    socket_addr: SocketAddr,
    sender: Sender<BackendMessage>,
    receiver: Receiver<FrontendMessage>,
) {
    let network_sender = NetworkMessageSender {
        // Add 1 to port so you can join and host on the same machine for testing
        socket: open_socket(socket_addr.port() + 1),
        destination: socket_addr,
    };
    println!("Joining {}", socket_addr);
    ping(network_sender.try_clone().unwrap(), socket_addr, sender);
    network_sender
        .send_network_message(NetworkMessage::JoinRequest)
        .unwrap();

    // Wait until we are accepted
    loop {
        // Check for new network requests
        let received_data = network_sender.socket.next_message();

        // Only continue when received data is not an error and a ClientAccepted message
        if let Some((NetworkMessage::ClientAccepted, origin)) = received_data {
            dbg!(&received_data);
            // Only break when origin is the host we want to connect to
            if origin == network_sender.destination {
                break;
            }
        }
    }
    println!("We were accepted");
}

fn ping(socket: NetworkMessageSender, socket_addr: SocketAddr, mut sender: Sender<BackendMessage>) {
    socket
        .send_network_message(NetworkMessage::Ping)
        .expect("Failed to send message");
    let now = std::time::Instant::now();
    // Some random NetworkMessage variant which isn't Ping
    let mut message = NetworkMessage::ClientAccepted;
    // Keep waiting for Ping with a 5 second timeout
    while message != NetworkMessage::Ping && now.elapsed() < Duration::from_secs(5) {
        let received_data = socket.socket.next_message();

        if received_data == None {
            continue;
        }
        let (received_message, received_origin) = received_data.unwrap();

        // Only accept ping when from the same origin
        if received_origin == socket_addr {
            message = received_message
        }
    }
    let ping_ms = now.elapsed().as_nanos() as f32 / 1_000_000 as f32;
    block_on(
        sender.send(BackendMessage::Client(crate::UIMessageClient::Ping(
            ping_ms,
        ))),
    )
    .unwrap();
}
