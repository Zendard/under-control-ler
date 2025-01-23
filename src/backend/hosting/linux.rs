use std::error::Error;

use evdev::{
    uinput::{VirtualDevice, VirtualDeviceBuilder},
    AbsInfo, AbsoluteAxisType, AttributeSet, BusType, EventType, InputEvent, InputId, Key,
    UinputAbsSetup,
};

use crate::{
    backend::{JOYSTICK_RANGE, TRIGGER_RANGE},
    AxisInput, ButtonInput, GamepadInput,
};

pub struct VirtualGamepad(VirtualDevice);

impl VirtualGamepad {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let mut keys = AttributeSet::new();

        // Add ABXY
        keys.insert(Key::BTN_NORTH);
        keys.insert(Key::BTN_EAST);
        keys.insert(Key::BTN_WEST);
        keys.insert(Key::BTN_SOUTH);

        // Add DPAD
        keys.insert(Key::BTN_DPAD_UP);
        keys.insert(Key::BTN_DPAD_DOWN);
        keys.insert(Key::BTN_DPAD_LEFT);
        keys.insert(Key::BTN_DPAD_RIGHT);

        // Add start and select
        keys.insert(Key::BTN_SELECT);
        keys.insert(Key::BTN_START);

        // Add bumpers
        keys.insert(Key::BTN_TL);
        keys.insert(Key::BTN_TR);

        // Add stick press
        keys.insert(Key::BTN_THUMBL);
        keys.insert(Key::BTN_THUMBR);

        // Magic numbers, found these somewhere online
        let stick_setup = AbsInfo::new(2293, -32768, 32767, 16, 128, 1);
        let trigger_setup = AbsInfo::new(2293, 0, 1023, 16, 128, 1);

        // Left stick
        let left_x = UinputAbsSetup::new(AbsoluteAxisType::ABS_X, stick_setup);
        let left_y = UinputAbsSetup::new(AbsoluteAxisType::ABS_Y, stick_setup);

        // Right stick
        let right_x = UinputAbsSetup::new(AbsoluteAxisType::ABS_RX, stick_setup);
        let right_y = UinputAbsSetup::new(AbsoluteAxisType::ABS_RY, stick_setup);

        // Triggers
        let left_trigger = UinputAbsSetup::new(AbsoluteAxisType::ABS_Z, trigger_setup);
        let right_trigger = UinputAbsSetup::new(AbsoluteAxisType::ABS_RZ, trigger_setup);

        let virtual_device = VirtualDeviceBuilder::new()?
            .name("Under Control(ler) Virtual Gamepad")
            // Numbers are vendor and client id, they are random
            .input_id(InputId::new(BusType::BUS_USB, 8629, 8629, 1))
            // Add list of keys to virtual gamepad
            .with_keys(&keys)?
            // Add sticks
            .with_absolute_axis(&left_x)?
            .with_absolute_axis(&left_y)?
            .with_absolute_axis(&right_x)?
            .with_absolute_axis(&right_y)?
            // Add triggers
            .with_absolute_axis(&left_trigger)?
            .with_absolute_axis(&right_trigger)?
            .build()?;
        Ok(Self(virtual_device))
    }

    pub fn play_input(&mut self, input: GamepadInput) {
        match input {
            GamepadInput::Axis(axis, value) => self.change_axis(axis, value),
            GamepadInput::Button(button, pressed) => self.change_button(button, pressed),
        }
    }

    fn change_button(&mut self, button: ButtonInput, pressed: bool) {
        // Value has to be numerical so true is 1, false is 0
        let value = match pressed {
            true => 1,
            false => 0,
        };

        let button = match button {
            ButtonInput::A => Key::BTN_SOUTH,
            ButtonInput::B => Key::BTN_EAST,
            ButtonInput::X => Key::BTN_NORTH,
            ButtonInput::Y => Key::BTN_WEST,
            ButtonInput::DpadUp => Key::BTN_DPAD_UP,
            ButtonInput::DpadDown => Key::BTN_DPAD_DOWN,
            ButtonInput::DpadLeft => Key::BTN_DPAD_LEFT,
            ButtonInput::DpadRight => Key::BTN_DPAD_RIGHT,
            ButtonInput::BumperLeft => Key::BTN_TL,
            ButtonInput::BumperRight => Key::BTN_TR,
            ButtonInput::StickLeft => Key::BTN_THUMBL,
            ButtonInput::StickRight => Key::BTN_THUMBR,
            ButtonInput::Select => Key::BTN_SELECT,
            ButtonInput::Start => Key::BTN_START,
        };

        let event = [InputEvent::new(EventType::KEY, button.code(), value)];
        self.0.emit(&event).unwrap()
    }
    fn change_axis(&mut self, axis: AxisInput, value: i8) {
        let value: i32 = if let AxisInput::TriggerLeft | AxisInput::TriggerRight = axis {
            value as i32 * TRIGGER_RANGE / 127
        } else if let AxisInput::StickLeftY | AxisInput::StickRightY = axis {
            value as i32 * -JOYSTICK_RANGE / 127
        } else {
            value as i32 * JOYSTICK_RANGE / 127
        };

        dbg!(&value);

        let axis = match axis {
            AxisInput::StickLeftX => AbsoluteAxisType::ABS_X,
            AxisInput::StickLeftY => AbsoluteAxisType::ABS_Y,
            AxisInput::StickRightX => AbsoluteAxisType::ABS_RX,
            AxisInput::StickRightY => AbsoluteAxisType::ABS_RY,
            AxisInput::TriggerLeft => AbsoluteAxisType::ABS_Z,
            AxisInput::TriggerRight => AbsoluteAxisType::ABS_RZ,
        };

        let event = [InputEvent::new(EventType::ABSOLUTE, axis.0, value)];
        self.0.emit(&event).unwrap()
    }
}
