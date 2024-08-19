use crate::events::InlineEvent;

pub struct StackWrapper {
    top_leaf: Option<TopLeaf>,
}

impl StackWrapper {
    pub fn new() -> Self {
        Self { top_leaf: None }
    }

    pub fn is_empty(&self) -> bool {
        self.top_leaf.is_none()
    }

    pub fn push_top_leaf(&mut self, entry: TopLeaf) {
        debug_assert!(self.top_leaf.is_none());

        self.top_leaf = Some(entry);
    }

    pub fn pop_top_leaf(&mut self) -> Option<TopLeaf> {
        self.top_leaf.take()
    }
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
    pub fn make_enter_event(&self) -> InlineEvent {
        InlineEvent::EnterCodeSpan
    }

    pub fn make_exit_event(&self) -> InlineEvent {
        InlineEvent::ExitInline
    }
}
