use bytes::{Buf, BufMut, Bytes, BytesMut};

use crate::{
    constants,
    packet::{PacketError, PacketKind},
};

#[derive(Debug)]
pub struct Packet {
    pub(crate) kind: PacketKind,
    pub(crate) sequence: u16,
    pub(crate) payload: Bytes,
}

impl Packet {
    pub fn kind(&self) -> &PacketKind {
        &self.kind
    }

    pub fn payload(&self) -> &[u8] {
        &self.payload
    }

    pub fn encode(&self) -> Bytes {
        let mut buffer =
            BytesMut::with_capacity(constants::PROTOCOL_HEADER_SIZE + self.payload.len());
        buffer.put_u8(self.kind as u8);
        buffer.put_u16(self.sequence);
        buffer.extend_from_slice(&self.payload);
        buffer.freeze()
    }

    pub fn decode(mut buf: &[u8]) -> Result<Self, PacketError> {
        if buf.len() < constants::PROTOCOL_HEADER_SIZE {
            return Err(PacketError::TooShort);
        }

        Ok(Self {
            kind: PacketKind::try_from(buf.get_u8())?,
            sequence: buf.get_u16(),
            payload: buf.copy_to_bytes(buf.remaining()),
        })
    }
}
