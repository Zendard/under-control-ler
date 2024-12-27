use std::net::{IpAddr, SocketAddr};

use iced::futures::channel::mpsc;
mod backend;
mod ui;

enum BackendMessage {
    Client(UIMessageClient),
    Server(UIMessageServer),
    Ready(mpsc::Sender<FrontendMessage>),
}

// This port number is just random, i hope it isn't used too often
pub const DEFAULT_PORT: u16 = 8629;

#[derive(Clone, Debug)]
enum UIMessageClient {
    Servers,
}

#[derive(Clone, Debug)]
enum UIMessageServer {
    JoinRequest(SocketAddr),
    ClientLeft(SocketAddr),
}

#[derive(Debug, PartialEq, Clone)]
enum FrontendMessage {
    StartHosting,
    StopHosting,
    Join(IpAddr),
    Leave,
}

fn main() -> iced::Result {
    iced::application("Under Control-ler", ui::State::update, ui::State::view)
        .theme(|_| iced::Theme::Oxocarbon)
        .subscription(ui::backend_subscription)
        .run()
}
