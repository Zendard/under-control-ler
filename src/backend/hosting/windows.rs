use vigem_client::{Client, TargetId, XButtons, XGamepad, Xbox360Wired};

use crate::{AxisInput, ButtonInput, GamepadInput};
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
            GamepadInput::Axis(axis, value) => self.change_axis(axis, value),
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
    fn change_axis(&mut self, axis: AxisInput, value: i8) {
        let value_stick = (value as i16) * 256;
        let value_trigger = (value as u8) << 1;
        match axis {
            AxisInput::StickLeftX => self.current_state.thumb_lx = value_stick,
            AxisInput::StickLeftY => self.current_state.thumb_ly = value_stick,
            AxisInput::StickRightX => self.current_state.thumb_rx = value_stick,
            AxisInput::StickRightY => self.current_state.thumb_ry = value_stick,
            AxisInput::TriggerLeft => self.current_state.left_trigger = value_trigger,
            AxisInput::TriggerRight => self.current_state.right_trigger = value_trigger,
        };
    }
}
