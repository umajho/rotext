#![cfg(feature = "block-id")]

use crate::types::BlockId;

pub struct BlockIdGenerator(usize);
impl BlockIdGenerator {
    pub fn new() -> Self {
        Self(0)
    }

    #[cfg(feature = "block-id")]
    pub fn pop(&mut self) -> BlockId {
        self.0 += 1;
        BlockId::new(self.0)
    }
}
