use crate::{
    events::ev,
    types::{BlockId, LineNumber, Tym},
    utils::{internal::array_queue::ArrayQueue, stack::Stack},
    Event,
};

#[cfg(feature = "block-id")]
use super::utils::BlockIdGenerator;
use super::{
    stack_wrapper::{StackEntry, StackWrapper},
    types::{CursorContext, YieldContext},
};

const MAX_TO_YIELD: usize = 5;

pub struct ParserInner<TStack: Stack<StackEntry>> {
    cursor: usize,

    current_line: LineNumber,

    pub stack: StackWrapper<TStack>,

    /// 承载的事件属于 `Block` 分组。
    to_yield: ArrayQueue<MAX_TO_YIELD, Event>,

    #[cfg(feature = "block-id")]
    block_id_generator: BlockIdGenerator,

    /// 记录仅在 [Parser] 的 `state` 为 `Expecting` 时才有效的数据。
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

    /// 返回的事件属于 `Block` 分组。
    pub fn pop_to_be_yielded(&mut self) -> Option<Event> {
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
    /// `ev` 是属于 `Block` 分组的事件。
    #[must_use]
    fn r#yield(&mut self, ev_to_yield: Event) -> Tym<1> {
        self.has_just_entered_table = matches!(ev_to_yield, ev!(Block, EnterTable(..)));

        self.to_yield.push_back(ev_to_yield);

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
