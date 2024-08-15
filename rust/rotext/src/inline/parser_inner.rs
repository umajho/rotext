use crate::{events::InlineEvent, types::Tym, utils::internal::array_queue::ArrayQueue};

use super::types::{CursorContext, YieldContext};

const MAX_TO_YIELD: usize = 2;

pub struct ParserInner {
    cursor: usize,

    to_yield: ArrayQueue<MAX_TO_YIELD, InlineEvent>,
}

impl ParserInner {
    pub fn new(cursor: usize) -> Self {
        Self {
            cursor,
            to_yield: ArrayQueue::new(),
        }
    }

    pub fn enforce_to_yield_mark(&self, _: Tym<MAX_TO_YIELD>) {}

    pub fn pop_to_be_yielded(&mut self) -> Option<InlineEvent> {
        self.to_yield.pop_front()
    }
}
impl CursorContext for ParserInner {
    fn cursor(&self) -> usize {
        self.cursor
    }

    fn move_cursor_forward(&mut self, n: usize) {
        self.cursor += n;
    }
}
impl YieldContext for ParserInner {
    #[must_use]
    fn r#yield(&mut self, ev: InlineEvent) -> Tym<1> {
        self.to_yield.push_back(ev);
        Tym::<1>::new()
    }
}
