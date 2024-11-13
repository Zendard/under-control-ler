use std::sync::{Arc, Mutex};

use iced::Element;
mod index;
mod join;
mod joined;

fn main() -> iced::Result {
    iced::application("Under Control-ler", App::update, App::view)
        .theme(|_| iced::Theme::Oxocarbon)
        .run()
}

#[derive(Default)]
struct App {
    screen: Screen,
}

#[derive(Debug, Clone)]
enum Message {
    Index(index::IndexMessage),
    Join(join::JoinMessage),
    Joined(joined::JoinedMessage),
}

enum Screen {
    Index(index::Index),
    Join(join::Join),
    Joined(joined::Joined),
    Host,
}

impl Default for Screen {
    fn default() -> Self {
        Screen::Index(index::Index)
    }
}

impl App {
    fn update(&mut self, message: Message) {
        match message {
            Message::Index(message) => match message {
                index::IndexMessage::Join => self.screen = Screen::Join(join::Join::default()),
                index::IndexMessage::Host => self.screen = Screen::Host,
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
        }
    }

    fn view(&self) -> Element<Message> {
        match &self.screen {
            Screen::Index(index) => index::Index::view(index),
            Screen::Join(join) => join::Join::view(join),
            Screen::Joined(joined) => joined::Joined::view(joined),
            Screen::Host => todo!(),
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
            *joined.stop.lock().unwrap() = true;
            self.screen = Screen::Index(index::Index)
        }
    }
}
