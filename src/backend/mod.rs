use std::{
    net::{Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket},
    string::ParseError,
};

pub mod hosting;
pub mod join;
const JOYSTICK_RANGE: isize = 32768;
const TRIGGER_RANGE: isize = 1023;
const NETWORK_BUFFER_SIZE: usize = 2;

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
}

impl NetworkMessageSocket {
    pub fn next_message(&self) -> Option<(NetworkMessage, SocketAddr)> {
        let mut recv_buf = [0; NETWORK_BUFFER_SIZE];
        let origin = self.0.recv_from(&mut recv_buf).ok()?.1;
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
            NetworkMessage::Ping => [0, 0],
            NetworkMessage::ClientAccepted => [1, 0],
        }
    }
}
impl TryFrom<[u8; NETWORK_BUFFER_SIZE]> for NetworkMessage {
    type Error = &'static str;
    fn try_from(buffer: [u8; NETWORK_BUFFER_SIZE]) -> Result<Self, Self::Error> {
        match buffer {
            [0, 0] => Ok(Self::Ping),
            [1, 0] => Ok(Self::ClientAccepted),
            _ => Err("Failed to parse NetworkMessage"),
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
}

fn open_socket(port: u16) -> NetworkMessageSocket {
    let udp_socket = UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), port))
        .expect("Failed to bind to port");
    NetworkMessageSocket(udp_socket)
}
