use crate::types::BlockId;

pub(super) fn count_continuous_character(input: &[u8], char: u8, since: usize) -> usize {
    let mut i = 0;
    while matches!(input.get(since+ i), Some(actual_char) if *actual_char == char) {
        i += 1;
    }

    i
}

pub(super) fn count_continuous_character_with_maximum(
    input: &[u8],
    char: u8,
    since: usize,
    maximum: usize,
) -> usize {
    let mut i = 0;
    while i < maximum && matches!(input.get(since + i), Some(actual_char) if *actual_char == char) {
        i += 1;
    }

    i
}

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
