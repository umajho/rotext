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

#[cfg(feature = "block-id")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockId(usize);
#[cfg(feature = "block-id")]
impl BlockId {
    pub fn new(id: usize) -> Self {
        Self(id)
    }
    pub fn value(&self) -> usize {
        self.0
    }
}
