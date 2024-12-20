use iced::{
    widget::{button, center, column, text},
    Element,
};

use super::{Screen, UIMessage};

pub fn view() -> Element<'static, super::UIMessage> {
    let title = center(text("Hosting...").size(30));
    let stop_button =
        center(button("Stop Hosting").on_press(UIMessage::ChangeScreen(Screen::Index)));
    center(column![title, stop_button].height(150).spacing(20)).into()
}
