use super::{host::HostScreen, Screen, ScreenTrait, UIMessage};
use iced::{
    widget::{button, center, column, text},
    Element,
};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct IndexScreen;

impl ScreenTrait for IndexScreen {
    fn view(&self) -> Element<'static, super::UIMessage> {
        let title = center(text("Under Control-ler").size(30));
        let join_button = center(button("Join"));
        let host_button = center(
            button("Host").on_press(UIMessage::ChangeScreen(Screen::Host(HostScreen::default()))),
        );
        center(
            column![title, join_button, host_button]
                .height(150)
                .spacing(20),
        )
        .into()
    }

    fn update(&mut self, _message: &UIMessage) {}
}
