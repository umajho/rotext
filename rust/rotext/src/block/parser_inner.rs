use crate::{
    events::BlockEvent,
    types::{BlockId, LineNumber},
    utils::{internal::array_queue::ArrayQueue, stack::Stack},
};

#[cfg(feature = "block-id")]
use super::utils::BlockIdGenerator;
use super::{
    stack_wrapper::{StackEntry, StackWrapper},
    types::{CursorContext, Tym, YieldContext},
};

const MAX_TO_YIELD: usize = 3;

pub struct ParserInner<TStack: Stack<StackEntry>> {
    cursor: usize,

    current_line: LineNumber,

    pub stack: StackWrapper<TStack>,

    to_yield: ArrayQueue<MAX_TO_YIELD, BlockEvent>,

    #[cfg(feature = "block-id")]
    block_id_generator: BlockIdGenerator,

    /// 记录仅在进入变体 `Expecting` 对应的分支时，在处理这个分支期间有效的数据。
    pub current_expecting: CurrentExpecting,

    has_just_entered_table: bool,
}

impl<TStack: Stack<StackEntry>> ParserInner<TStack> {
    pub fn new() -> Self {
        Self {
            cursor: 0,
            current_line: LineNumber::new_universal(1),
            stack: StackWrapper::new(),
            to_yield: ArrayQueue::new(),
            #[cfg(feature = "block-id")]
            block_id_generator: BlockIdGenerator::new(),
            current_expecting: CurrentExpecting::new(),
            has_just_entered_table: false,
        }
    }

    pub fn enforce_to_yield_mark(&self, _: Tym<MAX_TO_YIELD>) {}

    pub fn pop_to_be_yielded(&mut self) -> Option<BlockEvent> {
        self.to_yield.pop_front()
    }

    pub fn pop_block_id(&mut self) -> BlockId {
        #[cfg(feature = "block-id")]
        {
            self.block_id_generator.pop()
        }
        #[cfg(not(feature = "block-id"))]
        {
            BlockId()
        }
    }

    pub fn reset_current_expecting(&mut self) {
        self.current_expecting = CurrentExpecting::new();
    }

    pub fn has_just_entered_table(&mut self) -> bool {
        self.has_just_entered_table
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
        self.current_line.increase();
        self.stack.reset_current_line_for_new_line();
    }
}
impl<TStack: Stack<StackEntry>> YieldContext for ParserInner<TStack> {
    #[must_use]
    fn r#yield(&mut self, ev: BlockEvent) -> Tym<1> {
        self.has_just_entered_table = matches!(ev, BlockEvent::EnterTable(..));

        self.to_yield.push_back(ev);

        Tym::<1>::new()
    }
}

pub struct CurrentExpecting {
    spaces_before: usize,
}

impl CurrentExpecting {
    fn new() -> Self {
        Self { spaces_before: 0 }
    }

    pub fn spaces_before(&self) -> usize {
        self.spaces_before
    }

    pub fn set_spaces_before(&mut self, value: usize) {
        self.spaces_before = value;
    }
}
