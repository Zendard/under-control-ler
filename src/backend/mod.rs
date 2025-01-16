use crate::{ButtonInput, GamepadInput};
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket};

pub mod hosting;
pub mod join;
const JOYSTICK_RANGE: isize = 32768;
const TRIGGER_RANGE: isize = 1023;
const NETWORK_BUFFER_SIZE: usize = 4;

#[derive(Debug)]
pub struct NetworkMessageSocket(UdpSocket);

#[derive(Debug)]
pub struct NetworkMessageSender {
    socket: NetworkMessageSocket,
    destination: SocketAddr,
}

#[derive(Debug, PartialEq)]
pub enum NetworkMessage {
    Ping,
    ClientAccepted,
    JoinRequest,
    Input(GamepadInput),
}

impl NetworkMessageSocket {
    pub fn next_message(&self) -> Option<(NetworkMessage, SocketAddr)> {
        let mut recv_buf = [0; NETWORK_BUFFER_SIZE];
        let origin = self.0.recv_from(&mut recv_buf).ok()?.1;
        dbg!(&recv_buf);
        let message = recv_buf.try_into().ok()?;
        Some((message, origin))
    }

    pub fn try_clone(&self) -> std::io::Result<Self> {
        Ok(Self(self.0.try_clone()?))
    }
}

impl Into<[u8; NETWORK_BUFFER_SIZE]> for NetworkMessage {
    fn into(self) -> [u8; NETWORK_BUFFER_SIZE] {
        match self {
            NetworkMessage::Ping => [0, 0, 0, 0],
            NetworkMessage::JoinRequest => [1, 0, 0, 0],
            NetworkMessage::ClientAccepted => [1, 1, 0, 0],
            NetworkMessage::Input(gamepad_input) => gamepad_input.encode(),
        }
    }
}

impl GamepadInput {
    pub fn encode(self) -> [u8; NETWORK_BUFFER_SIZE] {
        match self {
            GamepadInput::Button(button_type, pressed) => [2, 0, button_type as u8, pressed.into()],
            GamepadInput::Axis(axis_type, axis_value) => [
                2,
                1,
                axis_type as u8,
                // Convert signed i8 from axis value into unsigned u8 by adding 128
                (axis_value as i16 + 128).try_into().unwrap(),
            ],
        }
    }
}

impl TryFrom<[u8; NETWORK_BUFFER_SIZE]> for NetworkMessage {
    type Error = &'static str;
    fn try_from(buffer: [u8; NETWORK_BUFFER_SIZE]) -> Result<Self, Self::Error> {
        match buffer {
            [0, 0, 0, 0] => Ok(Self::Ping),
            [1, 0, 0, 0] => Ok(Self::JoinRequest),
            [1, 1, 0, 0] => Ok(Self::ClientAccepted),
            [2, 0, button_type, pressed] => Ok(Self::Input(GamepadInput::Button(
                button_type.try_into().unwrap(),
                pressed == 1,
            ))),
            _ => Err("Failed to parse NetworkMessage"),
        }
    }
}

impl TryFrom<u8> for ButtonInput {
    type Error = &'static str;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::A),
            1 => Ok(Self::B),
            2 => Ok(Self::X),
            3 => Ok(Self::Y),
            4 => Ok(Self::DpadUp),
            5 => Ok(Self::DpadDown),
            6 => Ok(Self::DpadLeft),
            7 => Ok(Self::DpadRight),
            8 => Ok(Self::BumperLeft),
            9 => Ok(Self::BumperRight),
            10 => Ok(Self::StickLeft),
            11 => Ok(Self::StickRight),
            _ => Err("Failed to parse ButtonInput"),
        }
    }
}

type EasyResult<T> = Result<T, Box<dyn std::error::Error>>;
impl NetworkMessageSender {
    fn send_network_message(&self, message: NetworkMessage) -> EasyResult<()> {
        let message: [u8; NETWORK_BUFFER_SIZE] = message.into();
        self.socket.0.send_to(&message, self.destination)?;
        Ok(())
    }

    fn try_clone(&self) -> std::io::Result<Self> {
        Ok(NetworkMessageSender {
            socket: self.socket.try_clone()?,
            destination: self.destination.clone(),
        })
    }
}

fn open_socket(port: u16) -> NetworkMessageSocket {
    let udp_socket = UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), port))
        .expect("Failed to bind to port");
    NetworkMessageSocket(udp_socket)
}
