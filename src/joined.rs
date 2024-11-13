use std::sync::{Arc, Mutex};

use iced::{
    widget::{button, center, column, text},
    Element,
};

pub struct Joined {
    pub ip: String,
    pub stop: Arc<Mutex<bool>>,
}

#[derive(Clone, Debug)]
pub enum JoinedMessage {
    Leave,
}

impl Joined {
    pub fn view(&self) -> Element<crate::Message> {
        let title = center(text(format!("Connected to {}", self.ip)).size(30));
        let button = center(button("Leave").on_press(crate::Message::Joined(JoinedMessage::Leave)));
        center(column![title, button].height(150).spacing(20)).into()
    }
}
