use crate::{BackendMessage, FrontendMessage, UIMessageClient, UIMessageServer};
use iced::{
    futures::{channel::mpsc, SinkExt, Stream, StreamExt},
    stream, Element, Subscription,
};

mod host;
mod index;

#[derive(Debug)]
pub struct State {
    screen: Screen,
    mode: Mode,
    sender: mpsc::Sender<FrontendMessage>,
}
impl Default for State {
    fn default() -> Self {
        State {
            screen: Screen::Index,
            mode: Mode::None,
            sender: mpsc::channel(0).0,
        }
    }
}

#[derive(Debug, Default)]
enum Mode {
    Server,
    Client,
    #[default]
    None,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub enum Screen {
    #[default]
    Index,
    Host,
    // Join,
}

#[derive(Debug, Clone)]
pub enum UIMessage {
    Client(UIMessageClient),
    Server(UIMessageServer),
    ChangeScreen(Screen),
    Ready(mpsc::Sender<FrontendMessage>),
    None,
}

impl State {
    pub fn update(&mut self, message: UIMessage) {
        match message {
            UIMessage::ChangeScreen(screen) => self.change_screen(screen),
            UIMessage::Ready(sender) => self.sender = sender,
            _ => (),
        }
    }
    pub fn view(&self) -> Element<UIMessage> {
        match self.screen {
            Screen::Index => index::view(),
            Screen::Host => host::view(),
        }
    }

    fn change_screen(&mut self, screen: Screen) {
        let mut sender = self.sender.clone();
        if screen == Screen::Host {
            self.mode = Mode::Server;
            std::thread::spawn(|| async move {
                sender.send(FrontendMessage::StartHosting).await.unwrap();
            });
        } else if self.screen == Screen::Host && screen == Screen::Index {
            self.mode = Mode::None;
            std::thread::spawn(|| async move {
                sender.send(FrontendMessage::StopHosting).await.unwrap();
            });
        }
        self.screen = screen
    }
}

pub fn backend_subscription(state: &State) -> Subscription<UIMessage> {
    Subscription::run(subscription_worker).map(|backend_message| backend_message.into())
}

impl From<BackendMessage> for UIMessage {
    fn from(backend_message: BackendMessage) -> Self {
        match backend_message {
            BackendMessage::Client(message) => UIMessage::Client(message),
            BackendMessage::Server(message) => UIMessage::Server(message),
            BackendMessage::Ready(sender) => UIMessage::Ready(sender),
            _ => UIMessage::None,
        }
    }
}

fn subscription_worker() -> impl Stream<Item = BackendMessage> {
    stream::channel(100, |mut output| async move {
        let (ui_sender, backend_receiver) = mpsc::channel::<FrontendMessage>(100);
        let (backend_sender, ui_receiver) = mpsc::channel::<BackendMessage>(100);

        output.send(BackendMessage::Ready(ui_sender)).await.unwrap();

        std::thread::spawn(move || frontend_to_backend(backend_receiver, backend_sender));
        std::thread::spawn(move || backend_to_frontend(ui_receiver, output));
    })
}

async fn frontend_to_backend(
    mut backend_receiver: mpsc::Receiver<FrontendMessage>,
    backend_sender: mpsc::Sender<BackendMessage>,
) {
    loop {
        use iced::futures::StreamExt;
        let next_message = backend_receiver.select_next_some().await;
        match next_message {
            FrontendMessage::StartHosting => {
                crate::backend::hosting::host(backend_sender.clone(), backend_receiver)
            }
            _ => (),
        }
    }
}

async fn backend_to_frontend(
    mut ui_receiver: mpsc::Receiver<BackendMessage>,
    mut output: mpsc::Sender<BackendMessage>,
) {
    loop {
        let next_message = ui_receiver.select_next_some().await;
        output.send(next_message).await.unwrap();
    }
}
