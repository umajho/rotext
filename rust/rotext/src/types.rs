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
    pub fn new_universal(#[allow(unused_variables)] id: usize) -> Self {
        #[cfg(feature = "block-id")]
        {
            Self::new(id)
        }
        #[cfg(not(feature = "block-id"))]
        {
            Self::new()
        }
    }
    #[cfg(test)]
    pub fn new_invalid() -> Self {
        Self::new_universal(99999999)
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
    #[cfg(test)]
    pub fn new_invalid() -> Self {
        Self::new_universal(99999999)
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

/// Tym = To Yield Mark. 用于确保代码执行过程中不会爆 `to_yield` 栈的辅助类型。
pub struct Tym<const N: usize>;
pub const TYM_UNIT: Tym<0> = Tym::<0> {};
impl<const N: usize> Tym<N> {
    pub fn new() -> Self {
        Self
    }

    pub fn add<const M: usize>(self, _: Tym<M>) -> Tym<{ M + N }> {
        Tym::<{ M + N }>::new()
    }
}

/// 用于让 clippy 不去抱怨 useless conversion。
macro_rules! cast_tym {
    ($tym: expr) => {
        $tym.into()
    };
}
pub(super) use cast_tym;
macro_rules! impl_cast_tym {
    ($m:literal, $n:literal) => {
        impl From<Tym<$m>> for Tym<$n> {
            fn from(_: Tym<$m>) -> Self {
                Self
            }
        }
    };
}
impl_cast_tym!(0, 1);
impl_cast_tym!(0, 2);
impl_cast_tym!(0, 3);
impl_cast_tym!(1, 2);
impl_cast_tym!(1, 3);
impl_cast_tym!(2, 3);
