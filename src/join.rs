use iced::{
    widget::{button, center, column, text},
    Element,
};

pub struct Join;

#[derive(Clone, Debug)]
pub enum JoinMessage {
    Join,
    UpdateText,
}

impl Join {
    pub fn view(&self) -> Element<crate::Message> {
        let title = center(text("Join").size(30));
        center(column![title].height(150).spacing(20)).into()
    }

    pub fn update(&mut self, message: JoinMessage) {}
}
