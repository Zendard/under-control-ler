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

pub struct VirtualGamepad {
    injector: InputInjector,
    left_x: f64,
    left_y: f64,
    right_x: f64,
    right_y: f64,
}
unsafe impl Send for VirtualGamepad {}

impl VirtualGamepad {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let injector = InputInjector::TryCreate()?;
        injector.InitializeGamepadInjection()?;
        Ok(Self {
            injector,
            left_x: 0 as f64,
            left_y: 0 as f64,
            right_x: 0 as f64,
            right_y: 0 as f64,
        })
    }

    pub fn play_input(&mut self, input: GamepadInput) {
        match input {
            GamepadInput::Axis(axis, value) => self.change_axis(axis, value),
            GamepadInput::Button(button, pressed) => self.change_button(button, pressed),
        }
    }
    fn change_button(&mut self, button: ButtonInput, pressed: bool) {
        if !pressed {
            let button = GamepadButtons::None;
            let injected_input = InjectedInputGamepadInfo::new().unwrap();
            injected_input.SetButtons(button).ok();
            self.injector.InjectGamepadInput(&injected_input).unwrap();
            return;
        }

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
            ButtonInput::Select => GamepadButtons::View,
            ButtonInput::Start => GamepadButtons::Menu,
        };
        let injected_input = InjectedInputGamepadInfo::new().unwrap();
        injected_input.SetButtons(button).ok();
        self.injector.InjectGamepadInput(&injected_input).unwrap();
    }
    fn change_axis(&mut self, axis: AxisInput, value: i8) {
        let injected_input = InjectedInputGamepadInfo::new().unwrap();

        let value: f64 = value as f64 / 127 as f64;

        match axis {
            AxisInput::StickLeftX => self.left_x = value,
            AxisInput::StickLeftY => self.left_y = value,
            AxisInput::StickRightX => self.right_x = value,
            AxisInput::StickRightY => self.right_y = value,
            AxisInput::TriggerLeft => injected_input.SetLeftTrigger(value).unwrap(),
            AxisInput::TriggerRight => injected_input.SetRightTrigger(value).unwrap(),
        };

        injected_input.SetLeftThumbstickX(self.left_x).unwrap();
        injected_input.SetLeftThumbstickY(self.left_y).unwrap();

        injected_input.SetRightThumbstickX(self.right_x).unwrap();
        injected_input.SetRightThumbstickY(self.right_y).unwrap();

        self.injector.InjectGamepadInput(&injected_input).unwrap();
    }
}
