use std::sync::{Arc, Mutex};

use iced::{
    widget::{button, center, column, text},
    Element,
};

#[derive(Default)]
pub struct Host {
    pub clients: Vec<Client>,
    pub stop: Arc<Mutex<bool>>,
}

#[derive(Debug, Clone)]
pub struct Client {}

#[derive(Clone, Debug)]
pub enum HostMessage {
    Stop,
    ClientJoined(Client),
}

impl Host {
    pub fn view(&self) -> Element<crate::Message> {
        let title = center(text("Under Control-ler").size(30));
        let stop_button = button("Stop hosting").on_press(crate::Message::Host(HostMessage::Stop));
        center(column![title, stop_button].height(150).spacing(20)).into()
    }

    pub fn update(&mut self, message: HostMessage) {}

    pub fn stop(&self) {
        *self.stop.lock().unwrap() = true;
    }
}
