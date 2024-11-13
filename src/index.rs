use iced::{
    widget::{button, center, column, text},
    Element,
};

pub struct Index;

#[derive(Clone, Debug)]
pub enum IndexMessage {
    Join,
    Host,
}

impl Index {
    pub fn view(&self) -> Element<crate::Message> {
        let title = center(text("Under Control-ler").size(30));
        let join_button =
            center(button("Join").on_press(crate::Message::Index(IndexMessage::Join)));
        let host_button =
            center(button("Host").on_press(crate::Message::Index(IndexMessage::Host)));
        center(
            column![title, join_button, host_button]
                .height(150)
                .spacing(20),
        )
        .into()
    }
}
