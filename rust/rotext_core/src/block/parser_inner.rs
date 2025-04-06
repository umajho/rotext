use crate::{
    events::ev,
    internal_utils::array_queue::ArrayQueue,
    types::{BlockId, LineNumber, Stack, Tym},
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

#[derive(Debug, Clone)]
pub struct ParserInnerShallowSnapshot {
    cursor: usize,
    current_line: LineNumber,
    current_expecting: CurrentExpecting,
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

    /// 创建一个解析器的浅快照。这个快照只能在进入某个 top leaf 前创建才有用，且只能在进入的
    /// 那个 top leaf 退出时应用。用于名字中带 “potential” 的那些 top leaf 在发现其所期
    /// 待的构建并不成立时回退到先前的状态。
    pub fn take_shallow_snapshot(&self) -> ParserInnerShallowSnapshot {
        ParserInnerShallowSnapshot {
            cursor: self.cursor,
            current_line: self.current_line,
            current_expecting: self.current_expecting.clone(),
        }
    }

    pub fn apply_shallow_snapshot(&mut self, snapshot: ParserInnerShallowSnapshot) {
        self.cursor = snapshot.cursor;
        self.current_line = snapshot.current_line;
        self.current_expecting = snapshot.current_expecting;
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

    fn set_cursor(&mut self, cursor: usize) {
        self.cursor = cursor;
    }

    fn move_cursor_forward(&mut self, n: usize) {
        self.cursor += n;
    }

    fn current_line(&self) -> LineNumber {
        self.current_line
    }

    fn increase_current_line(&mut self, is_significant: bool) {
        self.current_line.increase();
        if is_significant {
            self.stack.reset_current_line_for_new_line();
        }
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

#[derive(Debug, Clone)]
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
