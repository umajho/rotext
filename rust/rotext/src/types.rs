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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LineNumber(#[cfg(feature = "line-number")] usize);
impl LineNumber {
    #[cfg(feature = "line-number")]
    pub fn new(n: usize) -> Self {
        Self(n)
    }
    #[cfg(not(feature = "line-number"))]
    pub fn new() -> Self {
        Self()
    }
    pub fn new_universal(#[allow(unused_variables)] n: usize) -> Self {
        #[cfg(feature = "line-number")]
        {
            Self::new(n)
        }
        #[cfg(not(feature = "line-number"))]
        {
            Self::new()
        }
    }

    #[cfg(feature = "line-number")]
    pub fn increase(&mut self) {
        self.0 += 1;
    }
    #[cfg(not(feature = "line-number"))]
    pub fn increase(&mut self) {}

    #[cfg(feature = "line-number")]
    pub fn value(&self) -> usize {
        self.0
    }
}
