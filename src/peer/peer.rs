use std::{
    io,
    net::{SocketAddr, ToSocketAddrs},
};

pub(crate) struct Peer {
    addr: SocketAddr,
}

impl Peer {
    pub const fn new(addr: SocketAddr) -> Self {
        Self { addr }
    }

    pub const fn addr(&self) -> SocketAddr {
        self.addr
    }

    pub fn resolve<A: ToSocketAddrs>(addr: A) -> io::Result<Self> {
        addr.to_socket_addrs()?
            .next()
            .map(|addr| Self { addr })
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Invalid address"))
    }
}
