use std::{
    io,
    net::{SocketAddr, ToSocketAddrs, UdpSocket},
};

pub struct UdpTransport<const BUFFER_SIZE: usize> {
    socket: UdpSocket,
    buffer: [u8; BUFFER_SIZE],
}

impl<const BUFFER_SIZE: usize> UdpTransport<BUFFER_SIZE> {
    pub fn new<A: ToSocketAddrs>(addr: A) -> io::Result<Self> {
        Ok(Self {
            socket: {
                let socket = UdpSocket::bind(addr)?;
                socket.set_nonblocking(true)?;
                socket
            },
            buffer: [0; BUFFER_SIZE],
        })
    }

    pub fn recv(&mut self) -> io::Result<(SocketAddr, &[u8])> {
        let (len, addr) = self.socket.recv_from(&mut self.buffer)?;
        Ok((addr, &self.buffer[..len]))
    }

    pub fn send(&self, addr: SocketAddr, data: impl AsRef<[u8]>) -> io::Result<()> {
        self.socket.send_to(data.as_ref(), addr)?;
        Ok(())
    }
}
