use crate::packet::Packet;

#[derive(Debug)]
pub enum ClientEvent {
    Connected,
    Disconnected,
    Data(Packet),
}
