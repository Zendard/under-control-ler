use iced::futures::channel::mpsc;
mod backend;
mod ui;

enum BackendMessage {
    Client(UIMessageClient),
    Server(UIMessageServer),
    Ready(mpsc::Sender<FrontendMessage>),
}

#[derive(Clone, Debug)]
enum UIMessageClient {
    Servers,
}

#[derive(Clone, Debug)]
enum UIMessageServer {
    JoinRequest,
    ClientLeft,
}

#[derive(Debug, PartialEq)]
enum FrontendMessage {
    StartHosting,
    StopHosting,
}

fn main() -> iced::Result {
    iced::application("Under Control-ler", ui::State::update, ui::State::view)
        .theme(|_| iced::Theme::Oxocarbon)
        .subscription(ui::backend_subscription)
        .run()
}
