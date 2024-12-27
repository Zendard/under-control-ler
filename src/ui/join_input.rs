use super::{host::HostScreen, Screen, ScreenTrait, UIMessage};
use iced::{
    widget::{button, center, column, text, text_input},
    Element,
};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct JoinInputScreen {
    ip: String,
}

impl ScreenTrait for JoinInputScreen {
    fn view(&self) -> Element<'static, super::UIMessage> {
        let title = center(text("Join").size(30));
        let ip_input = center(
            text_input("0.0.0.0", &self.ip)
                .width(150)
                .on_input(UIMessage::TextInput),
        );
        let join_button = center(button("Join"));
        center(
            column![title, ip_input, join_button]
                .height(150)
                .spacing(20),
        )
        .into()
    }

    fn update(&mut self, message: &UIMessage) {
        let message = message.clone();
        if let UIMessage::TextInput(text) = message {
            self.ip = text;
        }
    }
}
