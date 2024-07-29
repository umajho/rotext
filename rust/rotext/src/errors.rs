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
