use crate::{
    backend::{JOYSTICK_RANGE, TRIGGER_RANGE},
    AxisInput, ButtonInput, GamepadInput,
};
use std::error::Error;
use windows::{
    core::Param,
    Gaming::Input::GamepadButtons,
    UI::Input::Preview::Injection::{InjectedInputGamepadInfo, InputInjector},
};

pub struct VirtualGamepad(InputInjector);
unsafe impl Send for VirtualGamepad {}

impl VirtualGamepad {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let injector = InputInjector::TryCreate()?;
        injector.InitializeGamepadInjection()?;
        Ok(Self(injector))
    }

    pub fn play_input(&mut self, input: GamepadInput) {
        match input {
            GamepadInput::Axis(axis, value) => self.change_axis(axis, value),
            GamepadInput::Button(button, pressed) => self.change_button(button, pressed),
        }
    }
    fn change_button(&mut self, button: ButtonInput, pressed: bool) {
        let button = match button {
            ButtonInput::A => GamepadButtons::A,
            ButtonInput::B => GamepadButtons::B,
            ButtonInput::X => GamepadButtons::X,
            ButtonInput::Y => GamepadButtons::Y,
            ButtonInput::DpadUp => GamepadButtons::DPadUp,
            ButtonInput::DpadDown => GamepadButtons::DPadDown,
            ButtonInput::DpadLeft => GamepadButtons::DPadLeft,
            ButtonInput::DpadRight => GamepadButtons::DPadRight,
            ButtonInput::BumperLeft => GamepadButtons::LeftShoulder,
            ButtonInput::BumperRight => GamepadButtons::RightShoulder,
            ButtonInput::StickLeft => GamepadButtons::LeftThumbstick,
            ButtonInput::StickRight => GamepadButtons::RightThumbstick,
            ButtonInput::Select => GamepadButtons::Menu,
            ButtonInput::Start => GamepadButtons::View,
        };
        let injected_input = InjectedInputGamepadInfo::new().unwrap();
        injected_input.SetButtons(button).ok();
        self.0.InjectGamepadInput(&injected_input).unwrap();
    }
    fn change_axis(&mut self, axis: AxisInput, value: i8) {}
}
