use std::{
    collections::HashMap,
    io,
    net::{SocketAddr, ToSocketAddrs},
};

use bytes::Bytes;

use crate::{
    packet::{Packet, PacketKind},
    peer::{Peer, UdpTransport},
    server::{PeerId, ServerEvent},
};

pub struct ServerPeer {
    transport: UdpTransport,
    clients_by_addr: HashMap<SocketAddr, PeerId>,
    clients_by_id: HashMap<PeerId, Peer>,
    client_id_sequence: PeerId,
}

impl ServerPeer {
    pub fn new<A: ToSocketAddrs>(addr: A) -> io::Result<Self> {
        Ok(Self {
            transport: UdpTransport::new(addr)?,
            clients_by_addr: Default::default(),
            clients_by_id: Default::default(),
            client_id_sequence: PeerId(0),
        })
    }

    const fn next_id(&mut self) -> PeerId {
        let id = self.client_id_sequence;
        self.client_id_sequence = self.client_id_sequence.next();
        id
    }

    fn get_peer_id(&self, addr: SocketAddr) -> io::Result<PeerId> {
        match self.clients_by_addr.get(&addr) {
            Some(id) => Ok(*id),
            None => Err(io::Error::new(
                io::ErrorKind::NetworkUnreachable,
                "Client introuvable",
            )),
        }
    }

    fn get_peer_mut(&mut self, id: &PeerId) -> Option<&mut Peer> {
        self.clients_by_id.get_mut(id)
    }

    fn register_client(&mut self, addr: SocketAddr) -> Option<PeerId> {
        if !self.clients_by_addr.contains_key(&addr) {
            let id = self.next_id();
            self.clients_by_addr.insert(addr, id);
            self.clients_by_id.insert(id, Peer::new(addr));
            Some(id)
        } else {
            None
        }
    }

    fn recv(&mut self) -> io::Result<Option<(SocketAddr, Packet)>> {
        let Ok((addr, data)) = self.transport.recv() else {
            return Ok(None);
        };

        match Packet::decode(&data) {
            Ok(packet) => Ok(Some((addr, packet))),
            Err(e) => Err(io::Error::new(io::ErrorKind::InvalidData, e)),
        }
    }

    pub fn send(&mut self, kind: PacketKind, payload: Bytes) -> io::Result<()> {
        for peer in self.clients_by_id.values_mut() {
            let packet = peer.make_packet(kind, payload.clone());
            self.transport.send(peer.addr(), &packet.encode())?;
        }

        Ok(())
    }

    pub fn send_empty(&mut self, packet_kind: PacketKind) -> io::Result<()> {
        self.send(packet_kind, Bytes::new())
    }

    pub fn send_to(&mut self, id: PeerId, kind: PacketKind, payload: Bytes) -> io::Result<()> {
        match self.get_peer_mut(&id) {
            Some(peer) => {
                let packet = peer.make_packet(kind, payload);
                let addr = peer.addr();

                self.transport.send(addr, &packet.encode())
            }
            None => Err(io::Error::new(io::ErrorKind::NotFound, "Unknow peer")),
        }
    }

    pub fn send_empty_to(&mut self, id: PeerId, kind: PacketKind) -> io::Result<()> {
        self.send_to(id, kind, Bytes::new())
    }

    fn send_accept_to(&mut self, id: PeerId) -> io::Result<()> {
        self.send_empty_to(id, PacketKind::Accept)
    }

    pub fn poll(&mut self) -> io::Result<Option<ServerEvent>> {
        let Some((addr, packet)) = self.recv()? else {
            return Ok(None);
        };

        match &packet.kind {
            PacketKind::Request => {
                if let Some(id) = self.register_client(addr) {
                    println!("[ SERVER ] > Accept to {:?}", id);
                    self.send_accept_to(id)?;
                    Ok(Some(ServerEvent::NewClient(id)))
                } else {
                    Ok(None)
                }
            }
            PacketKind::Disconnect => {
                if let Some(id) = self.clients_by_addr.remove(&addr) {
                    println!("[ SERVER ] > Disconnect to {:?}", id);
                    self.send_empty_to(id, PacketKind::Disconnect)?;
                    self.clients_by_id.remove(&id);
                    Ok(Some(ServerEvent::DisconnectClient(id)))
                } else {
                    Ok(None)
                }
            }
            PacketKind::Ping => {
                let id = self.get_peer_id(addr)?;
                println!("[ SERVER ] > Pong to {:?}", id);
                self.send_empty_to(id, PacketKind::Pong)?;
                Ok(Some(ServerEvent::Data(id, packet)))
            }
            _ => {
                let id = self.get_peer_id(addr)?;
                Ok(Some(ServerEvent::Data(id, packet)))
            }
        }
    }
}
