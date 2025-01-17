use std::net::{IpAddr, SocketAddr};

use iced::futures::channel::mpsc;
mod backend;
mod ui;

// This port number is just random, i hope it isn't used too often
pub const DEFAULT_PORT: u16 = 8629;

#[derive(Debug, PartialEq)]
pub enum GamepadInput {
    Button(ButtonInput, bool),
    Axis(AxisInput, i8),
}

#[derive(Debug, PartialEq)]
pub enum ButtonInput {
    A,
    B,
    X,
    Y,
    DpadUp,
    DpadDown,
    DpadLeft,
    DpadRight,
    BumperLeft,
    BumperRight,
    StickLeft,
    StickRight,
    Select,
    Start,
}
#[derive(Debug, PartialEq)]
pub enum AxisInput {
    StickLeftX,
    StickLeftY,
    StickRightX,
    StickRightY,
    TriggerLeft,
    TriggerRight,
}

enum BackendMessage {
    Client(UIMessageClient),
    Server(UIMessageServer),
    Ready(mpsc::Sender<FrontendMessage>),
}

#[derive(Clone, Debug)]
enum UIMessageClient {
    Accepted,
    Ping(f32),
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
    AcceptClient(SocketAddr),
    Join(IpAddr),
    ChangeScreen(crate::ui::Screen),
    Leave,
}

fn main() -> iced::Result {
    iced::application("Under Control-ler", ui::State::update, ui::State::view)
        .theme(|_| iced::Theme::Oxocarbon)
        .subscription(ui::backend_subscription)
        .run()
}
