#[derive(Debug)]
pub enum ClientEvent<T> {
    Disconnect,
    Data(T),
}
