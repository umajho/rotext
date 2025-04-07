#[cfg(debug_assertions)]
use crate::events::is_event_of;
use crate::{Event, internal_utils::array_queue::ArrayQueue, types::Stack, types::Tym};

use super::{
    stack_wrapper::{StackEntry, StackWrapper},
    types::YieldContext,
};

const MAX_TO_YIELD: usize = 4;

pub struct ParserInner<TStack: Stack<StackEntry>> {
    pub stack: StackWrapper<TStack>,

    /// 承载的事件属于 `Inline` 分组。
    to_yield: ArrayQueue<MAX_TO_YIELD, Event>,

    /// XXX: 要确保 `cursor` 到达 `input.len()`，以让 `state` 变为 [State::Idle]。
    pub to_skip_input: ToSkipInputEvents,

    pub to_exit_until_popped_entry_from_stack: Option<StackEntry>,
}

impl<TStack: Stack<StackEntry>> ParserInner<TStack> {
    pub fn new() -> Self {
        Self {
            stack: StackWrapper::new(),
            to_yield: ArrayQueue::new(),
            to_skip_input: ToSkipInputEvents::default(),
            to_exit_until_popped_entry_from_stack: None,
        }
    }

    pub fn enforce_to_yield_mark(&self, _: Tym<MAX_TO_YIELD>) {}

    /// 返回的事件属于 `Inline` 分组。
    pub fn pop_to_be_yielded(&mut self) -> Option<Event> {
        self.to_yield.pop_front()
    }
}
impl<TStack: Stack<StackEntry>> YieldContext for ParserInner<TStack> {
    /// `ev` 是属于 `Inline` 分组的事件。
    #[must_use]
    fn r#yield(&mut self, ev: Event) -> Tym<1> {
        #[cfg(debug_assertions)]
        debug_assert!(is_event_of!(Inline, ev));
        self.to_yield.push_back(ev);
        Tym::<1>::new()
    }
}

#[derive(Default)]
pub struct ToSkipInputEvents {
    pub count: usize,
    /// 若非 `None`，则在 [count] 归零时（减去之后）断言下一个输入事件为
    /// [Event::__Unparsed]，并将 cursor 以指定的值初始化。
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
