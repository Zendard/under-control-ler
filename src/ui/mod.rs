use crate::{BackendMessage, FrontendMessage, UIMessageClient, UIMessageServer};
use host::HostScreen;
use iced::{
    futures::{channel::mpsc, executor::block_on, SinkExt, Stream, StreamExt},
    stream, Element, Subscription,
};

mod host;
mod index;

pub trait ScreenTrait {
    fn view(&self) -> Element<'static, UIMessage>;
    fn update(&mut self, message: &UIMessage);
}

#[derive(Debug)]
pub struct State {
    screen: Screen,
    mode: Mode,
    sender: mpsc::Sender<FrontendMessage>,
}
impl Default for State {
    fn default() -> Self {
        State {
            screen: Screen::default(),
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

#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    Index(index::IndexScreen),
    Host(host::HostScreen),
    // Join,
}

impl Default for Screen {
    fn default() -> Self {
        Screen::Index(index::IndexScreen)
    }
}

impl ScreenTrait for Screen {
    fn view(&self) -> Element<'static, UIMessage> {
        match self {
            Self::Host(screen) => screen.view(),
            Self::Index(screen) => screen.view(),
        }
    }
    fn update(&mut self, message: &UIMessage) {
        match self {
            Self::Host(screen) => screen.update(message),
            Self::Index(screen) => screen.update(message),
        }
    }
}

#[derive(Debug, Clone)]
pub enum UIMessage {
    Client(UIMessageClient),
    Server(UIMessageServer),
    ChangeScreen(Screen),
    Ready(mpsc::Sender<FrontendMessage>),
}

impl State {
    pub fn update(&mut self, message: UIMessage) {
        self.screen.update(&message);
        match message {
            UIMessage::ChangeScreen(screen) => self.change_screen(screen),
            UIMessage::Ready(sender) => {
                self.sender = sender;
            }
            _ => (),
        }
    }
    pub fn view(&self) -> Element<UIMessage> {
        self.screen.view()
    }

    fn change_screen(&mut self, screen: Screen) {
        let mut sender = self.sender.clone();
        match screen {
            Screen::Host(_) => {
                self.mode = Mode::Server;
                std::thread::spawn(move || {
                    block_on(sender.send(FrontendMessage::StartHosting)).unwrap();
                });
            }
            Screen::Index(_) => {
                if let Screen::Host(_) = self.screen {
                    self.mode = Mode::None;
                    std::thread::spawn(move || {
                        block_on(sender.send(FrontendMessage::StopHosting)).unwrap();
                    });
                }
            }
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
        }
    }
}

fn subscription_worker() -> impl Stream<Item = BackendMessage> {
    stream::channel(100, |mut output| async move {
        loop {
            // Create channels
            let (ui_sender, backend_receiver) = mpsc::channel::<FrontendMessage>(100);
            let (backend_sender, mut ui_receiver) = mpsc::channel::<BackendMessage>(100);
            // Send ui_sender to frontend
            output.send(BackendMessage::Ready(ui_sender)).await.unwrap();

            let handle =
                std::thread::spawn(move || frontend_to_backend(backend_receiver, backend_sender));

            // Pass BackendMessage to subscription stream
            loop {
                let next_message = ui_receiver.next().await;
                if let Some(next_message) = next_message {
                    output.send(next_message).await.unwrap();
                }
                // Break when hosting is stopped because backend_receiver is dropped
                if handle.is_finished() {
                    break;
                }
            }
        }
    })
}

fn frontend_to_backend(
    mut backend_receiver: mpsc::Receiver<FrontendMessage>,
    backend_sender: mpsc::Sender<BackendMessage>,
) {
    loop {
        let next_message = backend_receiver.try_next();
        if next_message.is_err() {
            continue;
        }

        if next_message.unwrap().unwrap() == FrontendMessage::StartHosting {
            crate::backend::hosting::host(
                crate::DEFAULT_PORT,
                backend_sender.clone(),
                backend_receiver,
            );
            break;
        }
    }
}
