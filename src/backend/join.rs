use std::net::SocketAddr;

use iced::futures::channel::mpsc::{Receiver, Sender};

use crate::{BackendMessage, FrontendMessage};

pub fn join(
    socket: SocketAddr,
    sender: Sender<BackendMessage>,
    receiver: Receiver<FrontendMessage>,
) {
    println!("Joining {}", socket);
}
