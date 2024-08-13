use crate::{types::LineNumber, BlockEvent};

/// 只提供读取与移动 cursor，以及读取及增加当前行数的上下文。
pub trait CursorContext {
    fn cursor(&self) -> usize;
    fn move_cursor_forward(&mut self, n: usize);

    fn current_line(&self) -> LineNumber;
    fn increase_current_line(&mut self);
}
pub trait YieldContext {
    #[must_use]
    fn r#yield(&mut self, ev: BlockEvent) -> Tym<1>;
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
