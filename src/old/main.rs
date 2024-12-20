use iced::{
    futures::{channel::mpsc, sink::SinkExt, Stream},
    stream, Element,
};
use std::sync::{
    mpsc::{Receiver, Sender},
    Arc, Mutex,
};
mod host;
mod index;
mod join;
mod joined;

fn main() -> iced::Result {
    iced::application("Under Control-ler", App::update, App::view)
        .theme(|_| iced::Theme::Oxocarbon)
        .run()
}

struct App {
    screen: Screen,
    receiver: Receiver<under_control_ler::Message>,
    sender: Sender<under_control_ler::Message>,
}

impl Default for App {
    fn default() -> Self {
        let (sender, receiver) = std::sync::mpsc::channel();
        Self {
            screen: Screen::Index(index::Index),
            receiver,
            sender,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    Index(index::IndexMessage),
    Join(join::JoinMessage),
    Joined(joined::JoinedMessage),
    Host(host::HostMessage),
    Ready(mpsc::Sender<under_control_ler::Message>),
}

enum Screen {
    Index(index::Index),
    Join(join::Join),
    Joined(joined::Joined),
    Host(host::Host),
}

impl App {
    fn update(&mut self, message: Message) {
        match message {
            Message::Index(message) => match message {
                index::IndexMessage::Join => self.screen = Screen::Join(join::Join::default()),
                index::IndexMessage::Host => self.host(),
            },
            Message::Join(message) => match message {
                join::JoinMessage::Join => self.join(),
                _ => {
                    if let Screen::Join(state) = &mut self.screen {
                        state.update(message)
                    }
                }
            },
            Message::Joined(message) => match message {
                joined::JoinedMessage::Leave => self.leave(),
            },
            Message::Host(message) => match message {
                host::HostMessage::Stop => self.stop_hosting(),
                _ => {
                    if let Screen::Host(state) = &mut self.screen {
                        state.update(message)
                    }
                }
            },
            Message::Ready(sender) => {
                std::thread::spawn(|| under_control_ler::host(8629, sender, receiver2));
            }
        }
    }

    fn view(&self) -> Element<Message> {
        match &self.screen {
            Screen::Index(index) => index::Index::view(index),
            Screen::Join(join) => join::Join::view(join),
            Screen::Joined(joined) => joined::Joined::view(joined),
            Screen::Host(host) => host::Host::view(host),
        }
    }

    fn join(&mut self) {
        if let Screen::Join(join) = &self.screen {
            let address = join.get_address().unwrap();
            let stop = Arc::new(Mutex::new(false));
            let stop_thread = stop.clone();
            let _ = std::thread::spawn(move || under_control_ler::join(address, stop_thread));
            self.screen = Screen::Joined(joined::Joined {
                ip: address.to_string(),
                stop,
            });
        }
    }

    fn leave(&mut self) {
        if let Screen::Joined(joined) = &self.screen {
            joined.leave();
            self.screen = Screen::Index(index::Index)
        }
    }

    fn host(&mut self) {
        if let Screen::Index(_) = &self.screen {
            let (sender2, receiver2) = std::sync::mpsc::channel();
            self.sender = sender2;
            self.screen = Screen::Host(host::Host {
                clients: Vec::new(),
            });
        }
    }

    fn stop_hosting(&mut self) {
        if let Screen::Host(host) = &self.screen {
            host.stop();
            self.screen = Screen::Index(index::Index)
        }
    }
}

fn channel_worker() -> impl Stream<Item = Message> {
    stream::channel(100, |mut output| async move {
        let (sender, mut receiver) = mpsc::channel(100);

        // Send the sender back to the application
        output.send(Message::Ready(sender)).await;

        loop {
            use iced::futures::StreamExt;

            // Read next input sent from `Application`
            let input = receiver.select_next_some().await;

            match input {
                under_control_ler::Message::ClientJoined(address) => {
                    output
                        .send(Message::Host(host::HostMessage::ClientJoined(address)))
                        .await;
                }
                _ => (),
            }
        }
    })
}
