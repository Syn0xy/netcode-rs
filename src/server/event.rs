use crate::peer::PeerId;

#[derive(Debug)]
pub enum ServerEvent<T> {
    NewConnection(PeerId),
    Disconnection(PeerId),
    Data(PeerId, T),
}
