#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct PeerId(u128);

impl PeerId {
    pub(crate) fn next(&mut self) -> Option<Self> {
        self.0.checked_add(1).map(|next_id| {
            let id = *self;
            self.0 = next_id;
            id
        })
    }
}

impl Default for PeerId {
    fn default() -> Self {
        Self(Default::default())
    }
}
