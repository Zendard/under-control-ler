use std::net::SocketAddr;

use crate::FrontendMessage;

use super::{index::IndexScreen, Screen, ScreenTrait, UIMessage, UIMessageServer};
use iced::{
    widget::{button, center, column, text},
    Element,
};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct HostScreen {
    current_requesting_client: Option<SocketAddr>,
    log_text: Vec<String>,
}

impl ScreenTrait for HostScreen {
    fn view(&self) -> Element<'static, super::UIMessage> {
        let title = center(text("Hosting...").size(30));
        let mut accept_button = None;
        if let Some(current_requesting_client) = self.current_requesting_client {
            accept_button = Some(center(button("Accept client").on_press(
                UIMessage::ToBackend(FrontendMessage::AcceptClient(current_requesting_client)),
            )));
        }
        let log_text = center(text(self.log_text.join("\n")));
        let stop_button = center(
            button("Stop Hosting").on_press(UIMessage::ChangeScreen(Screen::Index(IndexScreen))),
        );
        let mut content = column![];
        if let Some(_) = self.current_requesting_client {
            content = column![title, accept_button.unwrap(), log_text, stop_button];
        } else {
            content = column![title, log_text, stop_button];
        }
        center(content.height(200).spacing(20)).into()
    }

    fn update(&mut self, message: &UIMessage) {
        if let UIMessage::Server(message) = message {
            match message {
                UIMessageServer::JoinRequest(origin) => {
                    self.log_text.push(format!("{origin} wants to join"));
                    self.current_requesting_client = Some(*origin);
                }
                UIMessageServer::ClientLeft(origin) => {
                    self.log_text.push(format!("{origin} left"));
                }
            }
        } else if let UIMessage::ToBackend(FrontendMessage::AcceptClient(_)) = message {
            self.current_requesting_client = None;
        }
    }
}
