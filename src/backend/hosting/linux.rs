use crate::{backend::hosting::Client, BackendMessage, FrontendMessage};
use iced::futures::{channel::mpsc, StreamExt};
use std::net::{Ipv4Addr, SocketAddrV4, UdpSocket};

pub async fn host(
    port: u16,
    sender: mpsc::Sender<BackendMessage>,
    mut receiver: mpsc::Receiver<FrontendMessage>,
) {
    let accepted_clients: Vec<Client> = vec![];

    let socket = open_socket(port);

    loop {
        let message = receiver.select_next_some().await;
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
