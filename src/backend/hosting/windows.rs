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
    injected_input: InjectedInputGamepadInfo,
    current_buttons: GamepadButtons,
}
unsafe impl Send for VirtualGamepad {}

impl VirtualGamepad {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let injector = InputInjector::TryCreate()?;
        injector.InitializeGamepadInjection()?;
        Ok(Self {
            injector,
            injected_input: InjectedInputGamepadInfo::new()?,
            current_buttons: GamepadButtons::None,
        })
    }

    pub fn play_input(&mut self, input: GamepadInput) {
        match input {
            GamepadInput::Axis(axis, value) => self.change_axis(axis, value),
            GamepadInput::Button(button, pressed) => self.change_button(button, pressed),
        }
        self.injected_input
            .SetButtons(self.current_buttons)
            .unwrap();

        self.injector
            .InjectGamepadInput(&self.injected_input)
            .unwrap();
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
            ButtonInput::Select => GamepadButtons::View,
            ButtonInput::Start => GamepadButtons::Menu,
        };

        if pressed {
            self.current_buttons |= button;
        } else {
            self.current_buttons.0 ^= button.0;
        }
    }
    fn change_axis(&mut self, axis: AxisInput, value: i8) {
        let value: f64 = value as f64 / 127 as f64;

        match axis {
            AxisInput::StickLeftX => self.injected_input.SetLeftThumbstickX(value),
            AxisInput::StickLeftY => self.injected_input.SetLeftThumbstickY(value),
            AxisInput::StickRightX => self.injected_input.SetRightThumbstickX(value),
            AxisInput::StickRightY => self.injected_input.SetRightThumbstickY(value),
            AxisInput::TriggerLeft => self.injected_input.SetLeftTrigger(value),
            AxisInput::TriggerRight => self.injected_input.SetRightTrigger(value),
        }
        .unwrap();
    }
}
