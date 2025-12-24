use std::{error, fmt};

#[derive(Debug)]
pub enum PacketError {
    TooShort,
    InvalidKind(u8),
}

impl error::Error for PacketError {}

impl fmt::Display for PacketError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
