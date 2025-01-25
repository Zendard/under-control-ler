use vigem_client::{Client, TargetId, XButtons, XGamepad, Xbox360Wired};

use crate::{
    backend::{JOYSTICK_RANGE, TRIGGER_RANGE},
    AxisInput, ButtonInput, GamepadInput,
};
use std::error::Error;

pub struct VirtualGamepad {
    virtual_gamepad: Xbox360Wired<Client>,
    current_state: XGamepad,
}

impl VirtualGamepad {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let client = vigem_client::Client::connect()?;
        let mut gamepad = Xbox360Wired::new(client, TargetId::XBOX360_WIRED);
        gamepad.plugin()?;
        gamepad.wait_ready()?;
        Ok(VirtualGamepad {
            virtual_gamepad: gamepad,
            current_state: XGamepad::default(),
        })
    }

    pub fn play_input(&mut self, input: GamepadInput) {
        match input {
            GamepadInput::Axis(axis, value) => return,
            GamepadInput::Button(button, pressed) => self.change_button(button, pressed),
        };

        self.virtual_gamepad.update(&self.current_state).unwrap();
    }
    fn change_button(&mut self, button: ButtonInput, pressed: bool) {
        let button = match button {
            ButtonInput::A => XButtons::A,
            ButtonInput::B => XButtons::B,
            ButtonInput::X => XButtons::X,
            ButtonInput::Y => XButtons::Y,
            ButtonInput::DpadUp => XButtons::UP,
            ButtonInput::DpadDown => XButtons::DOWN,
            ButtonInput::DpadLeft => XButtons::LEFT,
            ButtonInput::DpadRight => XButtons::RIGHT,
            ButtonInput::BumperLeft => XButtons::LB,
            ButtonInput::BumperRight => XButtons::RB,
            ButtonInput::StickLeft => XButtons::LTHUMB,
            ButtonInput::StickRight => XButtons::RTHUMB,
            ButtonInput::Select => XButtons::BACK,
            ButtonInput::Start => XButtons::START,
        };

        if pressed {
            self.current_state.buttons.raw |= button;
        } else {
            self.current_state.buttons.raw &= !button;
        }
    }
    fn change_axis(&mut self, axis: AxisInput, value: i8) {}
}
