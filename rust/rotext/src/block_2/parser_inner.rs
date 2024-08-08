use crate::{
    events::BlockEvent,
    types::{BlockId, LineNumber},
    utils::{internal::array_queue::ArrayQueue, stack::Stack},
};

use super::{
    stack_wrapper::{StackEntry, StackWrapper},
    types::CursorContext,
    utils::BlockIdGenerator,
};

const MAX_TO_YIELD: usize = 3;

pub struct ParserInner<TStack: Stack<StackEntry>> {
    cursor: usize,

    current_line: LineNumber,

    pub stack: StackWrapper<TStack>,

    to_yield: ArrayQueue<MAX_TO_YIELD, BlockEvent>,

    #[cfg(feature = "block-id")]
    block_id_generator: BlockIdGenerator,
}

impl<TStack: Stack<StackEntry>> ParserInner<TStack> {
    pub fn new() -> Self {
        Self {
            cursor: 0,
            current_line: LineNumber::new_universal(1),
            stack: StackWrapper::new(),
            to_yield: ArrayQueue::new(),
            block_id_generator: BlockIdGenerator::new(),
        }
    }

    pub fn enforce_to_yield_mark(&self, _: Tym<MAX_TO_YIELD>) {}

    #[must_use]
    pub fn r#yield(&mut self, ev: BlockEvent) -> Tym<1> {
        self.to_yield.push_back(ev);

        Tym::<1>::new()
    }

    pub fn pop_to_be_yielded(&mut self) -> Option<BlockEvent> {
        self.to_yield.pop_front()
    }

    pub fn pop_block_id(&mut self) -> BlockId {
        self.block_id_generator.pop()
    }
}

impl<TStack: Stack<StackEntry>> CursorContext for ParserInner<TStack> {
    fn cursor(&self) -> usize {
        self.cursor
    }

    fn move_cursor_forward(&mut self, n: usize) {
        self.cursor += n;
    }

    fn current_line(&self) -> LineNumber {
        self.current_line
    }

    fn increase_current_line(&mut self) {
        self.current_line.increase()
    }
}

/// Tym = To Yield Mark. 用于确保代码执行过程中不会爆 `to_yield` 栈的辅助类型。
pub struct Tym<const N: usize>;
pub const TYM_UNIT: Tym<0> = Tym::<0> {};
impl<const N: usize> Tym<N> {
    fn new() -> Self {
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
