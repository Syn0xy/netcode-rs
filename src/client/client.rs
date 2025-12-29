use std::{
    io,
    net::ToSocketAddrs,
    time::{Duration, Instant},
};

use bytes::Bytes;
use log::{debug, info, trace, warn};

use crate::{
    client::{ClientEvent, ClientState},
    packet::{Packet, PacketKind},
    peer::{Peer, UdpTransport},
};

pub struct ClientPeer {
    state: ClientState,
    transport: UdpTransport,
    server: Peer,
}

impl ClientPeer {
    pub fn new<A: ToSocketAddrs, B: ToSocketAddrs>(addr: A, server_addr: B) -> io::Result<Self> {
        let server = Peer::resolve(server_addr)?;
        debug!("ClientPeer created, server={}", server.addr());

        Ok(Self {
            state: ClientState::Disconnected,
            transport: UdpTransport::new(addr)?,
            server,
        })
    }

    fn verify_can_recv(&self) -> io::Result<()> {
        match &self.state {
            ClientState::Connecting | ClientState::Connected => Ok(()),
            ClientState::Disconnected => Err(io::Error::new(
                io::ErrorKind::NotConnected,
                "Client not connected",
            )),
        }
    }

    fn verify_can_send(&self, kind: &PacketKind) -> io::Result<()> {
        match (&self.state, kind) {
            (ClientState::Connected, _) => Ok(()),
            (ClientState::Connecting, PacketKind::Request) => Ok(()),
            (ClientState::Connecting, _) | (ClientState::Disconnected, _) => Err(io::Error::new(
                io::ErrorKind::NotConnected,
                "Client not connected",
            )),
        }
    }

    fn recv(&mut self) -> io::Result<Option<Packet>> {
        self.verify_can_recv()?;

        let Ok((addr, data)) = self.transport.recv() else {
            return Ok(None);
        };

        if addr != self.server.addr() {
            return Ok(None);
        }

        match Packet::decode(&data) {
            Ok(packet) => Ok(Some(packet)),
            Err(e) => Err(io::Error::new(io::ErrorKind::InvalidData, e)),
        }
    }

    pub fn send(&mut self, kind: PacketKind, payload: Bytes) -> io::Result<()> {
        self.verify_can_send(&kind)?;

        let packet = self.server.make_packet(kind, payload);
        self.transport.send(self.server.addr(), &packet.encode())
    }

    pub fn send_empty(&mut self, kind: PacketKind) -> io::Result<()> {
        self.send(kind, Bytes::new())
    }

    fn send_request(&mut self) -> io::Result<()> {
        info!("Sending connection request to {}", self.server.addr());
        self.send_empty(PacketKind::Request)
    }

    pub fn poll(&mut self) -> io::Result<Option<ClientEvent>> {
        let Some(packet) = self.recv()? else {
            return Ok(None);
        };

        match (&self.state, &packet.kind) {
            (ClientState::Connecting, PacketKind::Accept) => {
                info!("Connected to server {}", self.server.addr());
                self.state = ClientState::Connected;
                return Ok(Some(ClientEvent::Connected));
            }
            (ClientState::Connected, PacketKind::Disconnect) => {
                info!("Disconnected by server {}", self.server.addr());
                self.state = ClientState::Disconnected;
                return Ok(Some(ClientEvent::Disconnected));
            }
            (_, PacketKind::Ping) => {
                trace!("Received ping, sending pong");
                self.send_empty(PacketKind::Pong)?;
            }
            _ => {}
        }

        Ok(Some(ClientEvent::Data(packet)))
    }

    pub fn connect(&mut self, timeout: Duration, interval: Duration) -> io::Result<()> {
        if self.state != ClientState::Disconnected {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "Connection already exists",
            ));
        }

        info!(
            "Connecting to {} (timeout={:?}, retry={:?})",
            self.server.addr(),
            timeout,
            interval
        );

        self.state = ClientState::Connecting;

        let start_time = Instant::now();
        let deadline = start_time + timeout;
        let mut next_retry = start_time;

        loop {
            let now = Instant::now();

            if now >= deadline {
                warn!("Connection to {} timed out", self.server.addr());
                self.state = ClientState::Disconnected;
                return Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    "Connection timeout",
                ));
            }

            if now >= next_retry {
                self.send_request()?;
                next_retry += interval;
            }

            if let Some(ClientEvent::Connected) = self.poll()? {
                return Ok(());
            }
        }
    }
}
