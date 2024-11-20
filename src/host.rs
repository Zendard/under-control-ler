use std::sync::{Arc, Mutex};

use iced::{
    widget::{button, center, column, scrollable, text, Column},
    Element,
};

pub struct Host {
    pub clients: Vec<Client>,
    pub stop: Arc<Mutex<bool>>,
    pub receiver: std::sync::mpsc::Receiver<under_control_ler::Message>,
}

#[derive(Debug, Clone)]
pub struct Client {
    ip: String,
}

#[derive(Clone, Debug)]
pub enum HostMessage {
    Stop,
    ClientJoined(Client),
}

impl Host {
    pub fn view(&self) -> Element<crate::Message> {
        let title = center(text(format!("{} clients connected", self.clients.len())).size(30));
        let stop_button =
            center(button("Stop hosting").on_press(crate::Message::Host(HostMessage::Stop)));

        let clients_vec = self
            .clients
            .iter()
            .map(|client| Element::new(text(client.ip.clone())))
            .collect();
        let clients = scrollable(Column::from_vec(clients_vec));
        center(column![title, clients, stop_button].height(150).spacing(20)).into()
    }

    pub fn update(&mut self, _message: HostMessage) {}

    pub fn stop(&self) {
        *self.stop.lock().unwrap() = true;
    }
}
