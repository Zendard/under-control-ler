use std::{
    net::{IpAddr, SocketAddr},
    str::FromStr,
};

use crate::{BackendMessage, FrontendMessage, UIMessageClient, UIMessageServer};
use host::HostScreen;
use iced::{
    futures::{channel::mpsc, executor::block_on, SinkExt, Stream, StreamExt},
    stream, Element, Subscription,
};

mod host;
mod index;
mod join;
mod join_input;

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
            screen: Screen::Index(index::IndexScreen),
            mode: Mode::None,
            // Initialize state with empty sender,
            // this is replaced with actual sender when you start hosting/joining
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
    JoinInput(join_input::JoinInputScreen),
    Join(join::JoinScreen),
}

// Pass ScreenTrait method calls to types inside enum cases
impl ScreenTrait for Screen {
    fn view(&self) -> Element<'static, UIMessage> {
        match self {
            Self::Host(screen) => screen.view(),
            Self::Index(screen) => screen.view(),
            Self::JoinInput(screen) => screen.view(),
            Self::Join(screen) => screen.view(),
        }
    }
    fn update(&mut self, message: &UIMessage) {
        match self {
            Self::Host(screen) => screen.update(message),
            Self::Index(screen) => screen.update(message),
            Self::JoinInput(screen) => screen.update(message),
            Self::Join(screen) => screen.update(message),
        }
    }
}

#[derive(Debug, Clone)]
pub enum UIMessage {
    // Message when you are a client (you joined someone)
    Client(UIMessageClient),
    // Message when you are hosting
    Server(UIMessageServer),
    // Message for changing screen
    ChangeScreen(Screen),
    // Message for updating text input
    TextInput(String),
    // Pass actual sender to state when hosting/joining
    Ready(mpsc::Sender<FrontendMessage>),
}

impl State {
    pub fn update(&mut self, message: UIMessage) {
        match message {
            UIMessage::ChangeScreen(screen) => self.change_screen(screen),
            // Set actual sender when hosting/joining
            UIMessage::Ready(sender) => {
                self.sender = sender;
            }
            // Pass message to screen logic
            _ => self.screen.update(&message),
        }
    }
    pub fn view(&self) -> Element<UIMessage> {
        // Let screen logic dictate UI
        self.screen.view()
    }

    fn change_screen(&mut self, screen: Screen) {
        let mut sender = self.sender.clone();
        match screen {
            Screen::Host(_) => {
                self.mode = Mode::Server;
                // Send StartHosting message to backend when changing to host screen
                std::thread::spawn(move || {
                    block_on(sender.send(FrontendMessage::StartHosting)).unwrap();
                });
            }
            Screen::Index(_) => {
                // Send StopHosting message to backend when changing to index from host screen
                if let Screen::Host(_) = self.screen {
                    self.mode = Mode::None;
                    std::thread::spawn(move || {
                        block_on(sender.send(FrontendMessage::StopHosting)).unwrap();
                    });
                }
                // Send Leave message to backend when changing to index from join screen
                else if let Screen::Join(_) = self.screen {
                    self.mode = Mode::None;
                    std::thread::spawn(move || {
                        block_on(sender.send(FrontendMessage::Leave)).unwrap();
                    });
                }
            }
            Screen::Join(_) => {
                if let Screen::JoinInput(join_input_state) = &self.screen {
                    let address =
                        IpAddr::from_str(&join_input_state.ip).expect("Failed to parse ip address");
                    self.mode = Mode::Client;
                    // Send Join message to backend when changing from JoinInput to Host screen
                    std::thread::spawn(move || {
                        block_on(sender.send(FrontendMessage::Join(address))).unwrap();
                    });
                }
            }
            _ => (),
        }
        // Set state screen to screen from arguments
        self.screen = screen
    }
}

// Subscription which listens to BackendMessages and converts to UIMessages
pub fn backend_subscription(_state: &State) -> Subscription<UIMessage> {
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
        // Check for next BackendMessage
        let message = backend_receiver.try_next();
        // Skip handling when message is an error
        if message.is_err() {
            continue;
        }
        let message = message.unwrap().unwrap();
        if message == FrontendMessage::StartHosting {
            crate::backend::hosting::host(
                crate::DEFAULT_PORT,
                backend_sender.clone(),
                backend_receiver,
            );
            // When host function is finished, we stopped hosting,
            // so we break to later reinitialize the backend sender and receiver
            break;
        }

        if let FrontendMessage::Join(ip) = message.clone() {
            crate::backend::join::join(
                SocketAddr::new(ip, crate::DEFAULT_PORT),
                backend_sender.clone(),
                backend_receiver,
            );
            // When join function is finished, we left,
            // so we break to later reinitialize the backend sender and receiver
            break;
        }
    }
}
