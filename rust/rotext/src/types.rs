pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    OutOfStackSpace,
}

impl Error {
    pub fn name(&self) -> &'static str {
        match self {
            Error::OutOfStackSpace => "OutOfStackSpace",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockId(#[cfg(feature = "block-id")] usize);
impl BlockId {
    #[cfg(feature = "block-id")]
    pub fn new(id: usize) -> Self {
        Self(id)
    }
    #[cfg(not(feature = "block-id"))]
    pub fn new() -> Self {
        Self()
    }

    #[cfg(feature = "block-id")]
    pub fn value(&self) -> usize {
        self.0
    }
}
