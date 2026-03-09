#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) enum Packet<T> {
    Request,
    Accept,
    Confirm,
    Disconnect,
    Data(T),
}
