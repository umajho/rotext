use crate::{
    events::InlineEvent,
    types::Tym,
    utils::{internal::array_queue::ArrayQueue, stack::Stack},
};

use super::{
    stack_wrapper::{StackEntry, StackWrapper},
    types::YieldContext,
};

const MAX_TO_YIELD: usize = 2;

pub struct ParserInner<TStack: Stack<StackEntry>> {
    pub stack: StackWrapper<TStack>,

    to_yield: ArrayQueue<MAX_TO_YIELD, InlineEvent>,

    should_skip_next_input_event: bool,
}

impl<TStack: Stack<StackEntry>> ParserInner<TStack> {
    pub fn new() -> Self {
        Self {
            stack: StackWrapper::new(),
            to_yield: ArrayQueue::new(),
            should_skip_next_input_event: false,
        }
    }

    pub fn enforce_to_yield_mark(&self, _: Tym<MAX_TO_YIELD>) {}

    pub fn pop_to_be_yielded(&mut self) -> Option<InlineEvent> {
        self.to_yield.pop_front()
    }

    pub fn set_should_skip_next_input_event(&mut self) {
        self.should_skip_next_input_event = true;
    }

    pub fn pop_should_skip_next_input_event(&mut self) -> bool {
        let ret = self.should_skip_next_input_event;
        self.should_skip_next_input_event = false;
        ret
    }
}
impl<TStack: Stack<StackEntry>> YieldContext for ParserInner<TStack> {
    #[must_use]
    fn r#yield(&mut self, ev: InlineEvent) -> Tym<1> {
        self.to_yield.push_back(ev);
        Tym::<1>::new()
    }
}
