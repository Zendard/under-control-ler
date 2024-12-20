use iced::{
    widget::{button, center, column, text, text_input},
    Element,
};
use std::{
    net::{IpAddr, SocketAddr},
    str::FromStr,
};
type Error = Box<dyn std::error::Error>;

#[derive(Default)]
pub struct Join {
    ip: String,
}

#[derive(Clone, Debug)]
pub enum JoinMessage {
    Join,
    IpChanged(String),
}

impl Join {
    pub fn view(&self) -> Element<crate::Message> {
        let title = center(text("Join").size(30));
        let ip_input = center(
            text_input("", &self.ip)
                .width(150)
                .on_input(|input| crate::Message::Join(JoinMessage::IpChanged(input))),
        );
        let button = center(button("Join").on_press(crate::Message::Join(JoinMessage::Join)));
        center(column![title, ip_input, button].height(150).spacing(20)).into()
    }

    pub fn update(&mut self, message: JoinMessage) {
        if let JoinMessage::IpChanged(content) = message {
            self.ip = content
        }
    }

    pub fn get_address(&self) -> Result<SocketAddr, Error> {
        Ok(SocketAddr::new(IpAddr::from_str(&self.ip)?, 8629))
    }
}
