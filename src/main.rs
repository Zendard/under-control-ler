use iced::Element;
mod index;
mod join;

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
}

enum Screen {
    Index(index::Index),
    Join(join::Join),
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
                index::IndexMessage::Join => self.screen = Screen::Join(join::Join),
                index::IndexMessage::Host => self.screen = Screen::Host,
            },
            Message::Join(message) => match message {
                join::JoinMessage::Join => self.screen = Screen::Host,
                _ => {
                    if let Screen::Join(state) = &mut self.screen {
                        state.update(message)
                    }
                }
            },
        }
    }
    fn view(&self) -> Element<Message> {
        match &self.screen {
            Screen::Index(index) => index::Index::view(&index),
            Screen::Join(join) => join::Join::view(&join),
            Screen::Host => todo!(),
        }
    }
}
