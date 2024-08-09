use crate::{
    events::{BlockEvent, ExitBlock, NewLine},
    types::{BlockId, LineNumber},
    utils::stack::Stack,
};

pub struct StackWrapper<TStack: Stack<StackEntry>> {
    stack: TStack,
    top_leaf: Option<TopLeaf>,

    item_likes_in_stack: usize,
    tables_in_stack: usize,

    last_accessed_item_like_at_current_line: LastAccessedItemLikeAtCurrentLine,
}

impl<TStack: Stack<StackEntry>> StackWrapper<TStack> {
    pub fn new() -> Self {
        Self {
            stack: TStack::new(),
            top_leaf: None,
            item_likes_in_stack: 0,
            last_accessed_item_like_at_current_line: LastAccessedItemLikeAtCurrentLine::new(),
            tables_in_stack: 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.top_leaf.is_none() && self.stack.as_slice().is_empty()
    }

    pub fn item_likes_in_stack(&self) -> usize {
        self.item_likes_in_stack
    }

    pub fn tables_in_stack(&self) -> usize {
        self.tables_in_stack
    }

    pub fn processed_item_likes_at_current_line(&self) -> usize {
        self.last_accessed_item_like_at_current_line.nth
    }

    pub fn has_unprocessed_item_likes_at_current_line(&self) -> bool {
        self.processed_item_likes_at_current_line() < self.item_likes_in_stack
    }

    pub fn first_unprocessed_item_like_at_current_line(&mut self) -> &StackEntryItemLikeContainer {
        self.update_last_accessed_item_like_next_index()
    }

    pub fn mark_first_unprocessed_item_like_as_processed_at_current_line(&mut self) {
        self.last_accessed_item_like_at_current_line.nth += 1;
        self.update_last_accessed_item_like_next_index();
        self.last_accessed_item_like_at_current_line.next_index += 1;
    }

    fn update_last_accessed_item_like_next_index(&mut self) -> &StackEntryItemLikeContainer {
        let slice = self.stack.as_slice();
        loop {
            match &slice[self.last_accessed_item_like_at_current_line.next_index] {
                StackEntry::ItemLikeContainer(stack_entry) => break stack_entry,
                _ => self.last_accessed_item_like_at_current_line.next_index += 1,
            }
        }
    }

    pub fn top_is_item_like_container(&self) -> bool {
        if self.top_leaf.is_some() {
            return false;
        }
        matches!(
            self.stack.as_slice().last(),
            Some(StackEntry::ItemLikeContainer(_))
        )
    }

    /// 向栈中推入一个 item-like entry。
    ///
    /// 调用者应保证 `self.top_leaf` 为 `None`。
    pub fn push_item_like(&mut self, stack_entry: StackEntryItemLike) -> crate::Result<()> {
        self.try_push(stack_entry.into())?;
        Ok(())
    }

    /// 向栈中推入一个 item-like 容器 entry。
    ///
    /// 调用者应保证 `self.top_leaf` 为 `None`。
    pub fn push_item_like_container(
        &mut self,
        stack_entry: StackEntryItemLikeContainer,
    ) -> crate::Result<()> {
        self.try_push(stack_entry.into())?;
        self.item_likes_in_stack += 1;
        Ok(())
    }

    /// 向栈中推入一个 table entry。
    ///
    /// 调用者应保证 `self.top_leaf` 为 `None`。
    pub fn push_table(&mut self, stack_entry: StackEntryTable) -> crate::Result<()> {
        self.try_push(stack_entry.into())?;
        self.tables_in_stack += 1;
        Ok(())
    }

    /// 尝试向栈中推入一个 entry。
    ///
    /// 调用者应保证 `self.top_leaf` 为 `None`。
    fn try_push(&mut self, entry: StackEntry) -> crate::Result<()> {
        debug_assert!(self.top_leaf.is_none());

        self.stack.try_push(entry)
    }

    pub fn push_top_leaf(&mut self, entry: TopLeaf) {
        debug_assert!(self.top_leaf.is_none());

        self.top_leaf = Some(entry);
    }

    /// 从栈中推出一个 entry。
    ///
    /// 调用者应保证 `self.top_leaf` 为 `None`。
    pub fn pop(&mut self) -> Option<StackEntry> {
        debug_assert!(self.top_leaf.is_none());

        let popped = self.stack.pop()?;

        match popped {
            StackEntry::ItemLike(_) => {}
            StackEntry::ItemLikeContainer(_) => self.item_likes_in_stack -= 1,
            StackEntry::Table(_) => self.tables_in_stack -= 1,
        }

        Some(popped)
    }

    pub fn pop_top_leaf(&mut self) -> Option<TopLeaf> {
        self.top_leaf.take()
    }
}

pub enum StackEntry {
    ItemLike(StackEntryItemLike),
    ItemLikeContainer(StackEntryItemLikeContainer),
    Table(StackEntryTable),
}
impl From<StackEntryItemLike> for StackEntry {
    fn from(value: StackEntryItemLike) -> Self {
        Self::ItemLike(value)
    }
}
impl From<StackEntryItemLikeContainer> for StackEntry {
    fn from(value: StackEntryItemLikeContainer) -> Self {
        Self::ItemLikeContainer(value)
    }
}
impl From<StackEntryTable> for StackEntry {
    fn from(value: StackEntryTable) -> Self {
        Self::Table(value)
    }
}

pub struct StackEntryItemLike {
    pub meta: Meta,

    pub r#type: GeneralItemLike,
}
impl StackEntryItemLike {
    pub fn make_enter_event(&self) -> BlockEvent {
        match self.r#type {
            GeneralItemLike::LI => BlockEvent::EnterListItem(self.meta.id.into()),
            GeneralItemLike::DT => BlockEvent::EnterDescriptionTerm(self.meta.id.into()),
            GeneralItemLike::DD => BlockEvent::EnterDescriptionDetails(self.meta.id.into()),
        }
    }

    pub fn make_exit_event(self, line_end: LineNumber) -> BlockEvent {
        self.meta.make_exit_event(line_end)
    }
}

pub struct StackEntryItemLikeContainer {
    pub meta: Meta,

    pub r#type: ItemLikeContainer,
}
impl StackEntryItemLikeContainer {
    pub fn make_enter_event(&self) -> BlockEvent {
        match self.r#type {
            ItemLikeContainer::BlockQuote => BlockEvent::EnterBlockQuote(self.meta.id.into()),
            ItemLikeContainer::OL => BlockEvent::EnterOrderedList(self.meta.id.into()),
            ItemLikeContainer::UL => BlockEvent::EnterUnorderedList(self.meta.id.into()),
            ItemLikeContainer::DL => BlockEvent::EnterDescriptionList(self.meta.id.into()),
        }
    }

    pub fn make_exit_event(self, line_end: LineNumber) -> BlockEvent {
        self.meta.make_exit_event(line_end)
    }
}

pub struct StackEntryTable {
    pub meta: Meta,
}
impl StackEntryTable {
    pub fn make_enter_event(&self) -> BlockEvent {
        BlockEvent::EnterTable(self.meta.id.into())
    }

    pub fn make_exit_event(self, line_end: LineNumber) -> BlockEvent {
        self.meta.make_exit_event(line_end)
    }
}

pub struct Meta {
    id: BlockId,
    line_start: LineNumber,
}
impl Meta {
    pub fn new(id: BlockId, line_start: LineNumber) -> Self {
        Self { id, line_start }
    }

    fn make_exit_event(self, line_end: LineNumber) -> BlockEvent {
        BlockEvent::ExitBlock(ExitBlock {
            id: self.id,
            start_line: self.line_start,
            end_line: line_end,
        })
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ItemLikeContainer {
    BlockQuote,
    /// ordered list
    OL,
    /// unordered list
    UL,
    /// description list
    DL,
}

#[derive(Clone, Copy)]
pub enum GeneralItemLike {
    /// of [ItemLikeContainer::OL] or [ItemLikeContainer::UL].
    LI,
    /// of [ItemLikeContainer::DL].
    DT,
    /// of [ItemLikeContainer::DL].
    DD,
}

pub enum TopLeaf {
    Paragraph(TopLeafParagraph),
    Heading(TopLeafHeading),
    CodeBlock(TopLeafCodeBlock),
}
impl From<TopLeafParagraph> for TopLeaf {
    fn from(value: TopLeafParagraph) -> Self {
        Self::Paragraph(value)
    }
}
impl From<TopLeafHeading> for TopLeaf {
    fn from(value: TopLeafHeading) -> Self {
        Self::Heading(value)
    }
}
impl From<TopLeafCodeBlock> for TopLeaf {
    fn from(value: TopLeafCodeBlock) -> Self {
        Self::CodeBlock(value)
    }
}

pub struct TopLeafParagraph {
    pub meta: Meta,

    pub new_line: Option<NewLine>,
}
impl TopLeafParagraph {
    pub fn make_enter_event(&self) -> BlockEvent {
        BlockEvent::EnterParagraph(self.meta.id.into())
    }

    pub fn make_exit_event(self, line_end: LineNumber) -> BlockEvent {
        self.meta.make_exit_event(line_end)
    }
}

pub struct TopLeafHeading {
    pub meta: Meta,

    pub level: usize,

    pub has_content_before: bool,
}
impl TopLeafHeading {
    pub fn make_enter_event(&self) -> BlockEvent {
        match self.level {
            1 => BlockEvent::EnterHeading1(self.meta.id.into()),
            2 => BlockEvent::EnterHeading2(self.meta.id.into()),
            3 => BlockEvent::EnterHeading3(self.meta.id.into()),
            4 => BlockEvent::EnterHeading4(self.meta.id.into()),
            5 => BlockEvent::EnterHeading5(self.meta.id.into()),
            6 => BlockEvent::EnterHeading6(self.meta.id.into()),
            _ => unreachable!(),
        }
    }

    pub fn make_exit_event(self, line_end: LineNumber) -> BlockEvent {
        self.meta.make_exit_event(line_end)
    }
}

pub struct TopLeafCodeBlock {
    pub meta: Meta,

    pub backticks: usize,

    pub state: TopLeafCodeBlockState,
}
pub enum TopLeafCodeBlockState {
    ExpectingInfoString,
    InInfoString,
    InCodeAtFirstLineBeginning,
    InCode,
}
impl TopLeafCodeBlock {
    pub fn make_enter_event(&self) -> BlockEvent {
        BlockEvent::EnterCodeBlock(self.meta.id.into())
    }

    pub fn make_exit_event(self, line_end: LineNumber) -> BlockEvent {
        self.meta.make_exit_event(line_end)
    }
}

struct LastAccessedItemLikeAtCurrentLine {
    nth: usize,
    next_index: usize,
}
impl LastAccessedItemLikeAtCurrentLine {
    fn new() -> Self {
        Self {
            nth: 0,
            next_index: 0,
        }
    }
}
