use std::{
    io,
    net::{SocketAddr, ToSocketAddrs},
};

use bytes::Bytes;

use crate::packet::{Packet, PacketKind};

pub struct Peer {
    addr: SocketAddr,
    next_sequence: u16,
}

impl Peer {
    pub fn new(addr: SocketAddr) -> Self {
        Self {
            addr,
            next_sequence: 0,
        }
    }

    pub fn resolve<A: ToSocketAddrs>(addr: A) -> io::Result<Self> {
        match addr.to_socket_addrs()?.next() {
            Some(socket_addr) => Ok(Self::new(socket_addr)),
            None => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Invalid server address",
            )),
        }
    }

    pub const fn addr(&self) -> SocketAddr {
        self.addr
    }

    const fn next_sequence(&mut self) -> u16 {
        let sequence = self.next_sequence;
        self.next_sequence = self.next_sequence.wrapping_add(1);
        sequence
    }

    pub const fn make_packet(&mut self, kind: PacketKind, payload: Bytes) -> Packet {
        Packet {
            kind,
            sequence: self.next_sequence(),
            payload,
        }
    }
}
