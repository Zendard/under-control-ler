use super::{index::IndexScreen, Screen, ScreenTrait, UIMessage, UIMessageServer};
use iced::{
    widget::{button, center, column, text},
    Element,
};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct HostScreen {
    log_text: Vec<String>,
}

impl ScreenTrait for HostScreen {
    fn view(&self) -> Element<'static, super::UIMessage> {
        let title = center(text("Hosting...").size(30));
        let log_text = center(text(self.log_text.join("\n")));
        let stop_button = center(button("Stop Hosting").on_press(UIMessage::ChangeScreen(
            Screen::Index(IndexScreen::default()),
        )));
        center(
            column![title, log_text, stop_button]
                .height(150)
                .spacing(20),
        )
        .into()
    }

    fn update(&mut self, message: &UIMessage) {
        if let UIMessage::Server(message) = message {
            match message {
                UIMessageServer::JoinRequest(origin) => {
                    self.log_text.push(format!("{origin} wants to join"))
                }
                UIMessageServer::ClientLeft(origin) => {
                    self.log_text.push(format!("{origin} left"));
                }
            }
        }
    }
}
