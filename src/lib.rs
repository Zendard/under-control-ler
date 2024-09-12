use std::{
    error::Error,
    net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket},
    str::FromStr,
    sync::{Arc, Mutex},
    thread,
};

#[cfg(target_os = "linux")]
use evdev::{
    uinput::{VirtualDevice, VirtualDeviceBuilder},
    AbsInfo, AbsoluteAxisType, AttributeSet, BusType, InputEvent, InputId, Key, UinputAbsSetup,
};
use gilrs::{ev::AxisOrBtn, Axis, Button, EventType, Gilrs};

pub struct JoinConfig {
    socket: SocketAddr,
}

impl JoinConfig {
    fn new(args: &[String]) -> JoinConfig {
        let ip_address = args.get(2).expect("Please enter an address");
        let ip_address = IpAddr::from_str(ip_address).expect("Please enter a valid ipv4 or ipv6");

        let default_port = "8629".to_string();
        let port = args.get(3).unwrap_or(&default_port);
        let port: u16 = port.parse().expect("Port number must be u16");

        let socket = SocketAddr::new(ip_address, port);

        JoinConfig { socket }
    }
}

pub struct HostConfig {
    pub port: u16,
}

impl HostConfig {
    fn new(args: &[String]) -> HostConfig {
        let default_port = "8629".to_string();
        let port = args.get(2).unwrap_or(&default_port);
        let port: u16 = port.parse().expect("Port number must be u16");

        HostConfig { port }
    }
}

#[cfg(target_os = "linux")]
struct VirtualGamepad(VirtualDevice);

const JOYSTICK_RANGE: isize = 32768;
const TRIGGER_RANGE: isize = 1023;

#[cfg(target_os = "linux")]
impl VirtualGamepad {
    fn new() -> Result<Self, Box<dyn Error>> {
        let mut keys = AttributeSet::new();
        keys.insert(Key::BTN_NORTH);
        keys.insert(Key::BTN_EAST);
        keys.insert(Key::BTN_WEST);
        keys.insert(Key::BTN_SOUTH);

        keys.insert(Key::BTN_DPAD_UP);
        keys.insert(Key::BTN_DPAD_DOWN);
        keys.insert(Key::BTN_DPAD_LEFT);
        keys.insert(Key::BTN_DPAD_RIGHT);

        keys.insert(Key::BTN_SELECT);
        keys.insert(Key::BTN_START);

        keys.insert(Key::BTN_TL);
        keys.insert(Key::BTN_TR);

        keys.insert(Key::BTN_THUMBL);
        keys.insert(Key::BTN_THUMBR);

        let abs_setup = AbsInfo::new(2293, -32768, 32767, 16, 128, 1);
        let trigger_setup = AbsInfo::new(2293, 0, 1023, 16, 128, 1);
        let left_x = UinputAbsSetup::new(AbsoluteAxisType::ABS_X, abs_setup);
        let left_y = UinputAbsSetup::new(AbsoluteAxisType::ABS_Y, abs_setup);

        let right_x = UinputAbsSetup::new(AbsoluteAxisType::ABS_RX, abs_setup);
        let right_y = UinputAbsSetup::new(AbsoluteAxisType::ABS_RY, abs_setup);

        let left_trigger = UinputAbsSetup::new(AbsoluteAxisType::ABS_Z, trigger_setup);
        let right_trigger = UinputAbsSetup::new(AbsoluteAxisType::ABS_RZ, trigger_setup);

        let gamepad = VirtualDeviceBuilder::new()?
            .name("Under Control(ler) Virtual Gamepad")
            .input_id(InputId::new(BusType::BUS_USB, 8629, 8629, 1))
            .with_keys(&keys)?
            .with_absolute_axis(&left_x)?
            .with_absolute_axis(&left_y)?
            .with_absolute_axis(&right_x)?
            .with_absolute_axis(&right_y)?
            .with_absolute_axis(&left_trigger)?
            .with_absolute_axis(&right_trigger)?
            .build()?;

        Ok(VirtualGamepad(gamepad))
    }

    fn set_key(&mut self, key: Key, value: i32) {
        self.0
            .emit(&[InputEvent::new(evdev::EventType::KEY, key.code(), value)])
            .unwrap()
    }

    fn set_axis(&mut self, axis: AbsoluteAxisType, value: f32) {
        let mut value = (value * JOYSTICK_RANGE as f32) as i32;

        if AbsoluteAxisType::ABS_Y == axis || AbsoluteAxisType::ABS_RY == axis {
            value = -value
        }

        self.0
            .emit(&[InputEvent::new(evdev::EventType::ABSOLUTE, axis.0, value)])
            .unwrap()
    }

    fn set_trigger(&mut self, axis: AbsoluteAxisType, value: f32) {
        let value = (value * TRIGGER_RANGE as f32) as i32;

        self.0
            .emit(&[InputEvent::new(evdev::EventType::ABSOLUTE, axis.0, value)])
            .unwrap()
    }
}

#[derive(Debug)]
struct RawMessage {
    data: [u8; 100],
    length: usize,
    _origin: SocketAddr,
}

pub fn join(args: &[String]) {
    let config = JoinConfig::new(args);

    let socket = make_connection(&config);
    send_controller_inputs(socket);
}

#[cfg(target_os = "linux")]
pub fn host(args: &[String]) {
    let config = HostConfig::new(args);

    open_port(&config);
}

#[cfg(target_os = "windows")]
pub fn host(args: &[String]) {
    println!("Hosting not yet possible on windows");
}

fn make_connection(join_config: &JoinConfig) -> UdpSocket {
    let socket = UdpSocket::bind(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0))
        .expect("Failed to bind socket");

    socket
        .connect(join_config.socket)
        .unwrap_or_else(|error| panic!("Failed to connect to {}: {}", join_config.socket, error));

    socket.send(b"Joined").unwrap();

    socket
}

fn send_controller_inputs(socket: UdpSocket) {
    let mut girls = Gilrs::new().unwrap();
    let gamepad_names = girls
        .gamepads()
        .map(|gamepad| gamepad.1.name().to_string())
        .collect::<Vec<String>>();

    println!("Detected gamepads: {:?}", gamepad_names);
    println!(
        "Connected to {}, sending inputs...",
        &socket.peer_addr().unwrap()
    );

    loop {
        handle_controller_event(&mut girls, &socket)
    }
}

fn handle_controller_event(girls: &mut Gilrs, socket: &UdpSocket) {
    while let Some(event) = girls.next_event() {
        if girls.gamepad(event.id).vendor_id() == Some(8629) {
            return;
        }

        let event = event.event;

        send_controller_event(event, socket);
    }
}

fn send_controller_event(event: EventType, socket: &UdpSocket) {
    let event_string = SimpleEventType(event).to_universal_string();
    let buffer: &[u8] = event_string.as_bytes();
    socket.send(buffer).unwrap_or_else(|error| {
        eprintln!("{}", error);
        0
    });
}

#[cfg(target_os = "linux")]
fn open_port(config: &HostConfig) {
    let socket = UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), config.port))
        .expect("Failed to bind to port");

    let gamepad = VirtualGamepad::new().unwrap();
    println!("Made virtual gamepad");
    let gamepad = Arc::new(Mutex::new(gamepad));

    let mut receive_buffer = [0; 100];
    println!("Waiting for inputs...");
    while let Ok((length, origin)) = socket.recv_from(&mut receive_buffer) {
        let data = receive_buffer;
        let message = RawMessage {
            data,
            length,
            _origin: origin,
        };
        let gamepad = Arc::clone(&gamepad);
        thread::spawn(move || handle_receive(message, gamepad));
    }
}

#[cfg(target_os = "linux")]
fn handle_receive(message: RawMessage, gamepad: Arc<Mutex<VirtualGamepad>>) {
    let data = &message.data[..message.length];
    let message_string = String::from_utf8(data.to_vec()).unwrap_or("Not valid utf-8".to_string());
    let event: Option<(AxisOrBtn, f32)> = SimpleEventType::from_string(&message_string);

    let Some(event) = event else { return };

    match event.0 {
        AxisOrBtn::Btn(button) => handle_button_changed(button, event.1, gamepad),
        AxisOrBtn::Axis(axis) => handle_axis_changed(axis, event.1, gamepad),
    }
}

#[derive(Debug)]
struct SimpleEventType(EventType);

impl SimpleEventType {
    fn to_universal_string(&self) -> String {
        match self.0 {
            EventType::ButtonChanged(button, value, _) => {
                format!("b,{},{}", serde_json::to_string(&button).unwrap(), value)
            }
            EventType::AxisChanged(axis, value, _) => {
                format!(
                    "a,{:?},{}",
                    serde_json::to_string(&axis)
                        .unwrap()
                        .replace("\"", "")
                        .replace("\\", ""),
                    value
                )
            }
            _ => "".to_string(),
        }
    }

    fn from_string(string: &str) -> Option<(AxisOrBtn, f32)> {
        let mut parts = string.split(',');
        let variant = parts.next()?;
        let name = parts.next()?;
        let value = parts.next()?.parse::<f32>().ok()?;

        match variant {
            "b" => Some((AxisOrBtn::Btn(serde_json::from_str(&name).ok()?), value)),
            "a" => Some((AxisOrBtn::Axis(serde_json::from_str(&name).ok()?), value)),
            _ => None,
        }
    }
}

#[cfg(target_os = "linux")]
fn handle_button_changed(button: Button, value: f32, gamepad: Arc<Mutex<VirtualGamepad>>) {
    let key = translate_button(button);

    match button {
        Button::LeftTrigger2 => gamepad
            .lock()
            .unwrap()
            .set_trigger(AbsoluteAxisType::ABS_Z, value),
        Button::RightTrigger2 => gamepad
            .lock()
            .unwrap()
            .set_trigger(AbsoluteAxisType::ABS_RZ, value),
        _ => {}
    };
    gamepad.lock().unwrap().set_key(key, value as i32);
}

#[cfg(target_os = "linux")]
fn translate_button(button: Button) -> Key {
    match button {
        Button::North => Key::BTN_NORTH,
        Button::East => Key::BTN_EAST,
        Button::West => Key::BTN_WEST,
        Button::South => Key::BTN_SOUTH,

        Button::DPadUp => Key::BTN_DPAD_UP,
        Button::DPadDown => Key::BTN_DPAD_DOWN,
        Button::DPadLeft => Key::BTN_DPAD_LEFT,
        Button::DPadRight => Key::BTN_DPAD_RIGHT,

        Button::Select => Key::BTN_SELECT,
        Button::Start => Key::BTN_START,

        Button::LeftTrigger => Key::BTN_TL,
        Button::RightTrigger => Key::BTN_TR,

        Button::LeftThumb => Key::BTN_THUMBL,
        Button::RightThumb => Key::BTN_THUMBR,

        _ => Key::BTN_NORTH,
    }
}

#[cfg(target_os = "linux")]
fn handle_axis_changed(axis: Axis, value: f32, gamepad: Arc<Mutex<VirtualGamepad>>) {
    let axis = translate_axis(axis);
    gamepad.lock().unwrap().set_axis(axis, value);
}

#[cfg(target_os = "linux")]
fn translate_axis(axis: Axis) -> AbsoluteAxisType {
    match axis {
        Axis::LeftStickX => AbsoluteAxisType::ABS_X,
        Axis::LeftStickY => AbsoluteAxisType::ABS_Y,

        Axis::RightStickX => AbsoluteAxisType::ABS_RX,
        Axis::RightStickY => AbsoluteAxisType::ABS_RY,

        _ => AbsoluteAxisType::ABS_X,
    }
}
