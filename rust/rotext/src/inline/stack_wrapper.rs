use crate::{events::ev, utils::stack::Stack, Event};

use super::types::EndCondition;

pub struct StackWrapper<TStack: Stack<StackEntry>> {
    stack: TStack,
    top_leaf: Option<TopLeaf>,
}

impl<TStack: Stack<StackEntry>> StackWrapper<TStack> {
    pub fn new() -> Self {
        Self {
            stack: TStack::new(),
            top_leaf: None,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.top_leaf.is_none() && self.stack.as_slice().is_empty()
    }

    pub fn push_entry(&mut self, entry: StackEntry) -> crate::Result<()> {
        debug_assert!(self.top_leaf.is_none());

        self.stack.try_push(entry)
    }

    pub fn push_top_leaf(&mut self, entry: TopLeaf) {
        debug_assert!(self.top_leaf.is_none());

        self.top_leaf = Some(entry);
    }

    pub fn pop_entry(&mut self) -> Option<StackEntry> {
        debug_assert!(self.top_leaf.is_none());

        self.stack.pop()
    }

    pub fn pop_top_leaf(&mut self) -> Option<TopLeaf> {
        self.top_leaf.take()
    }

    pub fn make_end_condition(&self) -> EndCondition {
        debug_assert!(self.top_leaf.is_none());

        let mut end_condition = EndCondition::default();

        match self.stack.as_slice().last() {
            Some(StackEntry::Strong) => {
                end_condition.on_strong_closing = true;
            }
            Some(StackEntry::Strikethrough) => {
                end_condition.on_strikethrough_closing = true;
            }
            Some(StackEntry::WikiLink) => {
                end_condition.on_wiki_link_closing = true;
            }
            None => {}
        }

        end_condition
    }
}

pub enum StackEntry {
    Strong,
    Strikethrough,
    WikiLink,
}

pub enum TopLeaf {
    CodeSpan(TopLeafCodeSpan),
}
impl From<TopLeafCodeSpan> for TopLeaf {
    fn from(value: TopLeafCodeSpan) -> Self {
        Self::CodeSpan(value)
    }
}

pub struct TopLeafCodeSpan {
    pub backticks: usize,
}
impl TopLeafCodeSpan {
    pub fn make_enter_event(&self) -> Event {
        ev!(Inline, EnterCodeSpan)
    }

    pub fn make_exit_event(&self) -> Event {
        ev!(Inline, ExitInline)
    }
}
