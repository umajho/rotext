use crate::{events::InlineEvent, utils::internal::array_queue::ArrayQueue};

const MAX_TO_YIELD: usize = 2;

pub struct ParserInner {
    to_yield: ArrayQueue<MAX_TO_YIELD, InlineEvent>,
}

impl ParserInner {
    pub fn new() -> Self {
        Self {
            to_yield: ArrayQueue::new(),
        }
    }

    pub fn r#yield(&mut self, ev: InlineEvent) {
        self.to_yield.push_back(ev);
    }

    pub fn pop_to_be_yielded(&mut self) -> Option<InlineEvent> {
        self.to_yield.pop_front()
    }
}
