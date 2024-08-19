use crate::types::BlockId;

pub struct BlockIdGenerator(#[cfg(feature = "block-id")] usize);
impl BlockIdGenerator {
    #[cfg(feature = "block-id")]
    pub fn new() -> Self {
        Self(0)
    }
    #[cfg(not(feature = "block-id"))]
    pub fn new() -> Self {
        Self()
    }

    #[cfg(feature = "block-id")]
    pub fn pop(&mut self) -> BlockId {
        self.0 += 1;
        BlockId::new(self.0)
    }
    #[cfg(not(feature = "block-id"))]
    pub fn pop(&mut self) -> BlockId {
        BlockId::new()
    }
}
