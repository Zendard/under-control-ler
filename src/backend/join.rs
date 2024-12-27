use super::{NetworkMessage, NetworkMessageSocket};
use crate::{backend::open_socket, BackendMessage, FrontendMessage};
use iced::futures::channel::mpsc::{Receiver, Sender};
use std::net::{SocketAddr, UdpSocket};

struct NetworkMessageSender {
    socket: NetworkMessageSocket,
    destination: SocketAddr,
}

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
    network_sender
        .send_network_message(NetworkMessage::Ping)
        .expect("Failed to send message");
}

type EasyResult<T> = Result<T, Box<dyn std::error::Error>>;
impl NetworkMessageSender {
    fn send_network_message(&self, message: NetworkMessage) -> EasyResult<()> {
        let message: [u8; 1] = message.into();
        self.socket.0.send_to(&message, self.destination)?;
        Ok(())
    }
}
