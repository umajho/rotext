#[cfg(feature = "block-id")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockID(usize);
#[cfg(feature = "block-id")]
impl BlockID {
    pub fn new(id: usize) -> Self {
        Self(id)
    }
    pub fn value(&self) -> usize {
        self.0
    }
}
