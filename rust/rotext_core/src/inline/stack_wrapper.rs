use crate::{Event, common::m, events::ev, types::Stack};

pub struct StackWrapper<TStack: Stack<StackEntry>> {
    stack: TStack,
    stack_entry_counts: StackEntryCounts,
    ruby_state: RubyState,
    leaf: Option<Leaf>,
}

impl<TStack: Stack<StackEntry>> StackWrapper<TStack> {
    pub fn new() -> Self {
        Self {
            stack: TStack::new(),
            stack_entry_counts: StackEntryCounts::default(),
            ruby_state: RubyState::default(),
            leaf: None,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.leaf.is_none() && self.stack.as_slice().is_empty()
    }

    pub fn push_entry(&mut self, entry: StackEntry) -> crate::Result<()> {
        debug_assert!(self.leaf.is_none());

        match entry {
            StackEntry::Emphasis => {
                self.stack_entry_counts.emphasis += 1;
            }
            StackEntry::Strong => {
                self.stack_entry_counts.strong += 1;
            }
            StackEntry::Strikethrough => {
                self.stack_entry_counts.strikethrough += 1;
            }
            StackEntry::WikiLink => {
                self.stack_entry_counts.wiki_link += 1;
            }
            StackEntry::_Ruby | StackEntry::_RubyText => {}
        }

        self.stack.try_push(entry)
    }

    pub fn is_in_ruby(&self) -> bool {
        self.ruby_state != RubyState::None
    }
    pub fn is_in_ruby_but_not_in_ruby_text(&self) -> bool {
        self.ruby_state == RubyState::Base
    }
    pub fn enter_ruby(&mut self) -> crate::Result<()> {
        debug_assert_eq!(self.ruby_state, RubyState::None);
        self.ruby_state = RubyState::Base;
        self.push_entry(StackEntry::_Ruby)
    }
    pub fn enter_ruby_text(&mut self) -> crate::Result<()> {
        debug_assert_eq!(self.ruby_state, RubyState::Base);
        self.ruby_state = RubyState::Text;
        self.push_entry(StackEntry::_RubyText)
    }
    pub fn exit_ruby(&mut self) {
        debug_assert_ne!(self.ruby_state, RubyState::None);
        self.ruby_state = RubyState::None;
    }

    pub fn push_leaf(&mut self, entry: Leaf) {
        debug_assert!(self.leaf.is_none());

        self.leaf = Some(entry);
    }

    pub fn pop_entry(&mut self) -> Option<StackEntry> {
        debug_assert!(self.leaf.is_none());

        let entry = self.stack.pop()?;
        match entry {
            StackEntry::Emphasis => {
                self.stack_entry_counts.emphasis -= 1;
            }
            StackEntry::Strong => {
                self.stack_entry_counts.strong -= 1;
            }
            StackEntry::Strikethrough => {
                self.stack_entry_counts.strikethrough -= 1;
            }
            StackEntry::WikiLink => {
                self.stack_entry_counts.wiki_link -= 1;
            }
            StackEntry::_Ruby => {
                self.exit_ruby();
            }
            StackEntry::_RubyText => {}
        }

        Some(entry)
    }

    pub fn pop_leaf(&mut self) -> Option<Leaf> {
        self.leaf.take()
    }

    pub fn make_end_condition(&self) -> EndCondition {
        debug_assert!(self.leaf.is_none());

        EndCondition {
            on_em_closing: self.stack_entry_counts.emphasis > 0,
            on_strong_closing: self.stack_entry_counts.strong > 0,
            on_strikethrough_closing: self.stack_entry_counts.strikethrough > 0,
            on_wiki_link_closing: self.stack_entry_counts.wiki_link > 0,

            on_ruby_closing: self.is_in_ruby(),
        }
    }
}

#[derive(PartialEq, Eq)]
pub enum StackEntry {
    Emphasis,
    Strong,
    Strikethrough,
    WikiLink,
    /// 应仅在本模块内直接使用。
    _Ruby,
    /// 应仅在本模块内直接使用。
    _RubyText,
}

#[derive(Default)]
struct StackEntryCounts {
    emphasis: usize,
    strong: usize,
    strikethrough: usize,
    wiki_link: usize,
}

#[derive(PartialEq, Eq, Default, Debug)]
enum RubyState {
    #[default]
    None,
    Base,
    Text,
}

pub enum Leaf {
    CodeSpan(LeafCodeSpan),
}
impl From<LeafCodeSpan> for Leaf {
    fn from(value: LeafCodeSpan) -> Self {
        Self::CodeSpan(value)
    }
}

pub struct LeafCodeSpan {
    pub backticks: usize,
}
impl LeafCodeSpan {
    pub fn make_enter_event(&self) -> Event {
        ev!(Inline, EnterCodeSpan)
    }

    pub fn make_exit_event(&self) -> Event {
        ev!(Inline, ExitInline)
    }
}

pub struct EndCondition {
    pub on_em_closing: bool,
    pub on_strong_closing: bool,
    pub on_strikethrough_closing: bool,
    pub on_wiki_link_closing: bool,

    pub on_ruby_closing: bool,
}

impl EndCondition {
    /// 若返回的栈的 entry 不为 None，则应该退出直至有一个该 entry 被弹出。
    pub fn test_1(&self, char: u8) -> Option<StackEntry> {
        if self.on_ruby_closing && char == m!(']') {
            Some(StackEntry::_Ruby)
        } else {
            None
        }
    }

    /// 若返回的栈的 entry 不为 None，则应该退出直至有一个该 entry 被弹出。
    pub fn test_2(&self, char: u8, char_next: Option<u8>) -> Option<StackEntry> {
        if char_next != Some(m!(']')) {
            return None;
        }

        if self.on_em_closing && char == m!('/') {
            Some(StackEntry::Emphasis)
        } else if self.on_strong_closing && char == m!('*') {
            Some(StackEntry::Strong)
        } else if self.on_strikethrough_closing && char == m!('~') {
            Some(StackEntry::Strikethrough)
        } else if self.on_wiki_link_closing && char == m!(']') {
            Some(StackEntry::WikiLink)
        } else {
            None
        }
    }
}
