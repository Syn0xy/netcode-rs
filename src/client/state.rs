#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ClientState {
    Disconnected,
    Connecting,
    Connected,
}
