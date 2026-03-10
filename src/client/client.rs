use std::{
    io,
    marker::PhantomData,
    net::ToSocketAddrs,
    time::{Duration, Instant},
};

use crate::{
    client::ClientEvent,
    constants::DEFAULT_CLIENT_BUFFER_SIZE,
    peer::{Packet, Peer, UdpTransport},
};

pub struct ClientPeer<T, const BUFFER_SIZE: usize = DEFAULT_CLIENT_BUFFER_SIZE> {
    transport: UdpTransport<BUFFER_SIZE>,
    server: Peer,
    phantom: PhantomData<T>,
}

impl<T, const BUFFER_SIZE: usize> ClientPeer<T, BUFFER_SIZE> {
    pub fn new<A, B>(addr: A, server_addr: B) -> io::Result<Self>
    where
        A: ToSocketAddrs,
        B: ToSocketAddrs,
    {
        Ok(Self {
            transport: UdpTransport::new(addr)?,
            server: Peer::resolve(server_addr)?,
            phantom: Default::default(),
        })
    }
}

impl<T: serde::Serialize, const BUFFER_SIZE: usize> ClientPeer<T, BUFFER_SIZE> {
    fn dispatch(&self, packet: Packet<T>) -> io::Result<()> {
        match postcard::to_allocvec(&packet) {
            Ok(data) => self.transport.send(self.server.addr(), data),
            Err(e) => Err(io::Error::new(io::ErrorKind::InvalidData, e)),
        }
    }

    pub fn send(&self, message: T) -> io::Result<()> {
        self.dispatch(Packet::Data(message))
    }

    pub fn disconnect(&self) -> io::Result<()> {
        self.dispatch(Packet::Disconnect)
    }
}

impl<T: serde::de::DeserializeOwned, const BUFFER_SIZE: usize> ClientPeer<T, BUFFER_SIZE> {
    fn recv(&mut self) -> io::Result<Option<Packet<T>>> {
        let Ok((addr, data)) = self.transport.recv() else {
            return Ok(None);
        };

        if addr != self.server.addr() {
            return Ok(None);
        }

        match postcard::from_bytes(data) {
            Ok(packet) => Ok(Some(packet)),
            Err(e) => Err(io::Error::new(io::ErrorKind::InvalidData, e)),
        }
    }

    pub fn poll(&mut self) -> io::Result<Option<ClientEvent<T>>> {
        let Some(packet) = self.recv()? else {
            return Ok(None);
        };

        Ok(match packet {
            Packet::Disconnect => Some(ClientEvent::Disconnect),
            Packet::Data(data) => Some(ClientEvent::Data(data)),
            _ => None,
        })
    }
}

impl<T: serde::Serialize + serde::de::DeserializeOwned, const BUFFER_SIZE: usize>
    ClientPeer<T, BUFFER_SIZE>
{
    pub fn connect(&mut self, timeout: Duration, interval: Duration) -> io::Result<()> {
        let start_time = Instant::now();
        let deadline = start_time + timeout;
        let mut next_retry = start_time;

        loop {
            let now = Instant::now();

            if now >= deadline {
                return Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    "Connection timeout",
                ));
            }

            if now >= next_retry {
                self.dispatch(Packet::Request)?;
                next_retry += interval;
            }

            if let Some(Packet::Accept) = self.recv()? {
                self.dispatch(Packet::Confirm)?;
                return Ok(());
            }
        }
    }
}
