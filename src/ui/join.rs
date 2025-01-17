use crate::UIMessageClient;

use super::{index::IndexScreen, Screen, ScreenTrait, UIMessage};
use iced::{
    widget::{button, center, column, scrollable, text, Column},
    Element,
};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct JoinScreen {
    log_text: Vec<String>,
}

impl ScreenTrait for JoinScreen {
    fn view(&self) -> Element<'static, super::UIMessage> {
        let title = center(text("Joining...").size(30));
        let log_text = center(scrollable(Column::from_vec(
            self.log_text
                .iter()
                .map(|log_message| text(log_message.clone()).into())
                .collect(),
        )))
        .height(100);
        let leave_button =
            center(button("Leave").on_press(UIMessage::ChangeScreen(Screen::Index(IndexScreen))));
        center(
            column![title, log_text, leave_button]
                .height(650)
                .spacing(20),
        )
        .into()
    }

    fn update(&mut self, message: &UIMessage) {
        if let UIMessage::Client(message) = message {
            let log_text = match message {
                UIMessageClient::Ping(delay) => format!("Ping: {delay}"),
                UIMessageClient::Accepted => "You were accepted by the host".to_string(),
            };
            self.log_text.push(log_text);
        }
    }
}
