use std::error::Error;

use evdev::{
    uinput::{VirtualDevice, VirtualDeviceBuilder},
    AbsInfo, AbsoluteAxisType, AttributeSet, BusType, InputId, Key, UinputAbsSetup,
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

        // Add triggers
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
}
