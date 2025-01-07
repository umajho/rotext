use crate::{common::m, events::ev, utils::stack::Stack, Event};

pub struct StackWrapper<TStack: Stack<StackEntry>> {
    stack: TStack,
    stack_entry_counts: StackEntryCounts,
    top_leaf: Option<TopLeaf>,
}

impl<TStack: Stack<StackEntry>> StackWrapper<TStack> {
    pub fn new() -> Self {
        Self {
            stack: TStack::new(),
            stack_entry_counts: StackEntryCounts::default(),
            top_leaf: None,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.top_leaf.is_none() && self.stack.as_slice().is_empty()
    }

    pub fn push_entry(&mut self, entry: StackEntry) -> crate::Result<()> {
        debug_assert!(self.top_leaf.is_none());

        match entry {
            StackEntry::Strong => {
                self.stack_entry_counts.strong += 1;
            }
            StackEntry::Strikethrough => {
                self.stack_entry_counts.strikethrough += 1;
            }
            StackEntry::WikiLink => {
                self.stack_entry_counts.wiki_link += 1;
            }
        }

        self.stack.try_push(entry)
    }

    pub fn push_top_leaf(&mut self, entry: TopLeaf) {
        debug_assert!(self.top_leaf.is_none());

        self.top_leaf = Some(entry);
    }

    pub fn pop_entry(&mut self) -> Option<StackEntry> {
        debug_assert!(self.top_leaf.is_none());

        let entry = self.stack.pop()?;
        match entry {
            StackEntry::Strong => {
                self.stack_entry_counts.strong -= 1;
            }
            StackEntry::Strikethrough => {
                self.stack_entry_counts.strikethrough -= 1;
            }
            StackEntry::WikiLink => {
                self.stack_entry_counts.wiki_link -= 1;
            }
        }

        Some(entry)
    }

    pub fn pop_top_leaf(&mut self) -> Option<TopLeaf> {
        self.top_leaf.take()
    }

    pub fn make_end_condition(&self) -> EndCondition {
        debug_assert!(self.top_leaf.is_none());

        EndCondition {
            on_strong_closing: self.stack_entry_counts.strong > 0,
            on_strikethrough_closing: self.stack_entry_counts.strikethrough > 0,
            on_wiki_link_closing: self.stack_entry_counts.wiki_link > 0,
        }
    }
}

#[derive(PartialEq, Eq)]
pub enum StackEntry {
    Strong,
    Strikethrough,
    WikiLink,
}

#[derive(Default)]
struct StackEntryCounts {
    strong: usize,
    strikethrough: usize,
    wiki_link: usize,
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

pub struct EndCondition {
    pub on_strong_closing: bool,
    pub on_strikethrough_closing: bool,
    pub on_wiki_link_closing: bool,
}

impl EndCondition {
    /// 若返回的栈的 entry 不为 None，则应该退出直至有一个该 entry 被弹出。
    pub fn test(&self, char: u8, char_next: Option<u8>) -> Option<StackEntry> {
        if self.on_strong_closing && char == m!('\'') && char_next == Some(m!(']')) {
            Some(StackEntry::Strong)
        } else if self.on_strikethrough_closing && char == m!('~') && char_next == Some(m!(']')) {
            Some(StackEntry::Strikethrough)
        } else if self.on_wiki_link_closing && char == m!(']') && char_next == Some(m!(']')) {
            Some(StackEntry::WikiLink)
        } else {
            None
        }
    }
}
