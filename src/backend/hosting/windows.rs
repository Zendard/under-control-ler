use crate::{
    backend::{JOYSTICK_RANGE, TRIGGER_RANGE},
    AxisInput, ButtonInput, GamepadInput,
};
use std::error::Error;

pub struct VirtualGamepad;

impl VirtualGamepad {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        Ok(Self)
    }

    pub fn play_input(&mut self, input: GamepadInput) {}
    fn change_button(&mut self, button: ButtonInput, pressed: bool) {}
    fn change_axis(&mut self, axis: AxisInput, value: i8) {}
}
