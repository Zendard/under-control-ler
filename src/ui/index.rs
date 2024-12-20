use iced::{
    widget::{button, center, column, text},
    Element,
};

use super::{Screen, UIMessage};

pub fn view() -> Element<'static, super::UIMessage> {
    let title = center(text("Under Control-ler").size(30));
    let join_button = center(button("Join"));
    let host_button = center(button("Host").on_press(UIMessage::ChangeScreen(Screen::Host)));
    center(
        column![title, join_button, host_button]
            .height(150)
            .spacing(20),
    )
    .into()
}
