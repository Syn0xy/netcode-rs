#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct PeerId(pub(crate) u128);

impl PeerId {
    pub(crate) const fn next(&self) -> Self {
        Self(self.0 + 1)
    }
}
