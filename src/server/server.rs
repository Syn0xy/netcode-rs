use std::{
    collections::HashMap,
    io,
    marker::PhantomData,
    net::{SocketAddr, ToSocketAddrs},
};

use crate::{
    constants::DEFAULT_SERVER_BUFFER_SIZE,
    peer::{Packet, Peer, PeerId, UdpTransport},
    server::ServerEvent,
};

pub struct ServerPeer<T, const BUFFER_SIZE: usize = DEFAULT_SERVER_BUFFER_SIZE> {
    transport: UdpTransport<BUFFER_SIZE>,
    client_id_sequence: PeerId,
    clients_by_addr: HashMap<SocketAddr, PeerId>,
    clients_by_id: HashMap<PeerId, Peer>,
    phantom: PhantomData<T>,
}

impl<T, const BUFFER_SIZE: usize> ServerPeer<T, BUFFER_SIZE> {
    pub fn new<A: ToSocketAddrs>(addr: A) -> io::Result<Self> {
        Ok(Self {
            transport: UdpTransport::new(addr)?,
            client_id_sequence: Default::default(),
            clients_by_addr: Default::default(),
            clients_by_id: Default::default(),
            phantom: Default::default(),
        })
    }

    fn next_client_id(&mut self) -> Option<PeerId> {
        self.client_id_sequence.next()
    }
}

impl<T: serde::Serialize, const BUFFER_SIZE: usize> ServerPeer<T, BUFFER_SIZE> {
    fn dispatch(&self, addr: SocketAddr, packet: Packet<T>) -> io::Result<()> {
        match postcard::to_allocvec(&packet) {
            Ok(data) => self.transport.send(addr, data),
            Err(e) => Err(io::Error::new(io::ErrorKind::InvalidData, e)),
        }
    }

    pub fn send_to(&self, id: &PeerId, message: T) -> io::Result<()> {
        match self.clients_by_id.get(id) {
            Some(peer) => self.dispatch(peer.addr(), Packet::Data(message)),
            None => Err(io::Error::new(io::ErrorKind::NotFound, "Unknow peer")),
        }
    }

    pub fn broadcast(&self, message: T) -> io::Result<()> {
        let packet = Packet::Data(message);

        match postcard::to_allocvec(&packet) {
            Ok(data) => {
                for peer in self.clients_by_id.values() {
                    self.transport.send(peer.addr(), &data)?;
                }
                Ok(())
            }
            Err(e) => Err(io::Error::new(io::ErrorKind::InvalidData, e)),
        }
    }
}

impl<T: serde::de::DeserializeOwned, const BUFFER_SIZE: usize> ServerPeer<T, BUFFER_SIZE> {
    fn recv(&mut self) -> io::Result<Option<(SocketAddr, Packet<T>)>> {
        let Ok((addr, data)) = self.transport.recv() else {
            return Ok(None);
        };

        match postcard::from_bytes(data) {
            Ok(packet) => Ok(Some((addr, packet))),
            Err(e) => Err(io::Error::new(io::ErrorKind::InvalidData, e)),
        }
    }
}

impl<T: serde::Serialize + serde::de::DeserializeOwned, const BUFFER_SIZE: usize>
    ServerPeer<T, BUFFER_SIZE>
{
    pub fn poll(&mut self) -> io::Result<Option<ServerEvent<T>>> {
        let Some((addr, packet)) = self.recv()? else {
            return Ok(None);
        };

        if let Packet::Request = packet {
            if self.clients_by_addr.contains_key(&addr) {
                return Ok(None);
            }

            self.dispatch(addr, Packet::Accept)?;

            let Some(peer_id) = self.next_client_id() else {
                return Ok(None);
            };

            self.clients_by_addr.insert(addr, peer_id);
            self.clients_by_id.insert(peer_id, Peer::new(addr));

            return Ok(None);
        }

        let Some(&peer_id) = self.clients_by_addr.get(&addr) else {
            return Ok(None);
        };

        Ok(match packet {
            Packet::Confirm => Some(ServerEvent::NewConnection(peer_id)),
            Packet::Disconnect => Some(ServerEvent::Disconnection(peer_id)),
            Packet::Data(data) => Some(ServerEvent::Data(peer_id, data)),
            _ => None,
        })
    }
}
