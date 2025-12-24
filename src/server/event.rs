use crate::{packet::Packet, server::PeerId};

#[derive(Debug)]
pub enum ServerEvent {
    NewClient(PeerId),
    DisconnectClient(PeerId),
    Data(PeerId, Packet),
}
