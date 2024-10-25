use crate::{
    events::InlineEvent,
    types::Tym,
    utils::{internal::array_queue::ArrayQueue, stack::Stack},
};

use super::{
    stack_wrapper::{StackEntry, StackWrapper},
    types::YieldContext,
};

const MAX_TO_YIELD: usize = 4;

pub struct ParserInner<TStack: Stack<StackEntry>> {
    pub stack: StackWrapper<TStack>,

    to_yield: ArrayQueue<MAX_TO_YIELD, InlineEvent>,

    /// XXX: 要确保 `cursor` 到达 `input.len()`，以让 `state` 变为 [State::Idle]。
    pub to_skip_input: ToSkipInputEvents,
}

impl<TStack: Stack<StackEntry>> ParserInner<TStack> {
    pub fn new() -> Self {
        Self {
            stack: StackWrapper::new(),
            to_yield: ArrayQueue::new(),
            to_skip_input: ToSkipInputEvents::default(),
        }
    }

    pub fn enforce_to_yield_mark(&self, _: Tym<MAX_TO_YIELD>) {}

    pub fn pop_to_be_yielded(&mut self) -> Option<InlineEvent> {
        self.to_yield.pop_front()
    }
}
impl<TStack: Stack<StackEntry>> YieldContext for ParserInner<TStack> {
    #[must_use]
    fn r#yield(&mut self, ev: InlineEvent) -> Tym<1> {
        self.to_yield.push_back(ev);
        Tym::<1>::new()
    }
}

#[derive(Default)]
pub struct ToSkipInputEvents {
    pub count: usize,
    /// 若非 `None`，则在 [count] 归零时（减去之后）断言下一个输入事件为
    /// [InlineInputEvent::Unparsed]，并将 cursor 以指定的值初始化。
    pub cursor_value: Option<usize>,
}

impl ToSkipInputEvents {
    pub fn new_one() -> Self {
        Self {
            count: 1,
            cursor_value: None,
        }
    }
}
