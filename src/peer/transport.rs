use std::{
    io,
    net::{SocketAddr, ToSocketAddrs, UdpSocket},
};

use log::debug;

use crate::constants;

pub struct UdpTransport {
    socket: UdpSocket,
    buffer: [u8; constants::MAX_PACKET_SIZE],
}

impl UdpTransport {
    pub fn new<A: ToSocketAddrs>(addr: A) -> io::Result<Self> {
        let socket = UdpSocket::bind(addr)?;

        socket.set_nonblocking(true)?;

        debug!("UDP transport bound to {:?}", socket.peer_addr());

        Ok(Self {
            socket,
            buffer: [0; constants::MAX_PACKET_SIZE],
        })
    }

    pub fn recv(&mut self) -> io::Result<(SocketAddr, &[u8])> {
        let (len, addr) = self.socket.recv_from(&mut self.buffer)?;
        Ok((addr, &self.buffer[..len]))
    }

    pub fn send<D: AsRef<[u8]>>(&self, addr: SocketAddr, data: D) -> io::Result<()> {
        self.socket.send_to(data.as_ref(), addr)?;
        Ok(())
    }
}
