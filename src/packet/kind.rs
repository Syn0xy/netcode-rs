use crate::packet::PacketError;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PacketKind {
    Request = 1,
    Accept = 2,
    Disconnect = 3,
    Ping = 4,
    Pong = 5,
    Data = 6,
}

impl TryFrom<u8> for PacketKind {
    type Error = PacketError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(PacketKind::Request),
            2 => Ok(PacketKind::Accept),
            3 => Ok(PacketKind::Disconnect),
            4 => Ok(PacketKind::Ping),
            5 => Ok(PacketKind::Pong),
            6 => Ok(PacketKind::Data),
            _ => Err(PacketError::InvalidKind(value)),
        }
    }
}
