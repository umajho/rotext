use crate::{
    events::{ev, ExitBlock, NewLine},
    types::{BlockId, LineNumber},
    utils::stack::Stack,
    Event,
};

pub struct StackWrapper<TStack: Stack<StackEntry>> {
    stack: TStack,
    top_leaf: Option<TopLeaf>,

    item_likes_in_stack: usize,
    tables_in_stack: usize,

    should_reset_state: bool,
}

impl<TStack: Stack<StackEntry>> StackWrapper<TStack> {
    pub fn new() -> Self {
        Self {
            stack: TStack::new(),
            top_leaf: None,
            item_likes_in_stack: 0,
            tables_in_stack: 0,
            should_reset_state: false,
        }
    }

    pub fn as_slice(&self) -> &[StackEntry] {
        self.stack.as_slice()
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

    pub fn reset_current_line_for_new_line(&mut self) {
        self.should_reset_state = true;
    }

    pub fn should_reset_state(&self) -> bool {
        self.should_reset_state
    }

    pub fn reset_should_reset_state(&mut self) {
        self.should_reset_state = false;
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

    pub fn top_is_description_term(&self) -> bool {
        if self.top_leaf.is_some() {
            return false;
        }
        matches!(
            self.stack.as_slice().last(),
            Some(StackEntry::ItemLike(StackEntryItemLike {
                r#type: GeneralItemLike::DT,
                ..
            }))
        )
    }

    pub fn top_is_table(&self) -> bool {
        if self.top_leaf.is_some() {
            return false;
        }
        matches!(self.stack.as_slice().last(), Some(StackEntry::Table(_)))
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

    pub fn is_top_leaf_some(&self) -> bool {
        self.top_leaf.is_some()
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
    /// 返回的事件属于 `Block` 分组。
    pub fn make_enter_event(&self) -> Event {
        match self.r#type {
            GeneralItemLike::LI => ev!(Block, EnterListItem(self.meta.id.into())),
            GeneralItemLike::DT => ev!(Block, EnterDescriptionTerm(self.meta.id.into())),
            GeneralItemLike::DD => ev!(Block, EnterDescriptionDetails(self.meta.id.into())),
        }
    }

    /// 返回的事件属于 `Block` 分组。
    pub fn make_exit_event(self, line_end: LineNumber) -> Event {
        self.meta.make_exit_event(line_end)
    }
}

pub struct StackEntryItemLikeContainer {
    pub meta: Meta,

    pub r#type: ItemLikeContainer,
}
impl StackEntryItemLikeContainer {
    /// 返回的事件属于 `Block` 分组。
    pub fn make_enter_event(&self) -> Event {
        match self.r#type {
            ItemLikeContainer::BlockQuote => ev!(Block, EnterBlockQuote(self.meta.id.into())),
            ItemLikeContainer::OL => ev!(Block, EnterOrderedList(self.meta.id.into())),
            ItemLikeContainer::UL => ev!(Block, EnterUnorderedList(self.meta.id.into())),
            ItemLikeContainer::DL => ev!(Block, EnterDescriptionList(self.meta.id.into())),
        }
    }

    /// 返回的事件属于 `Block` 分组。
    pub fn make_exit_event(self, line_end: LineNumber) -> Event {
        self.meta.make_exit_event(line_end)
    }
}

pub struct StackEntryTable {
    pub meta: Meta,
}
impl StackEntryTable {
    /// 返回的事件属于 `Block` 分组。
    pub fn make_enter_event(&self) -> Event {
        ev!(Block, EnterTable(self.meta.id.into()))
    }

    /// 返回的事件属于 `Block` 分组。
    pub fn make_exit_event(self, line_end: LineNumber) -> Event {
        self.meta.make_exit_event(line_end)
    }
}

#[derive(Debug)]
pub struct Meta {
    id: BlockId,
    line_start: LineNumber,
}
impl Meta {
    pub fn new(id: BlockId, line_start: LineNumber) -> Self {
        Self { id, line_start }
    }

    /// 返回的事件属于 `Block` 分组。
    fn make_exit_event(self, line_end: LineNumber) -> Event {
        ev!(
            Block,
            ExitBlock(ExitBlock {
                id: self.id,
                start_line: self.line_start,
                end_line: line_end,
            })
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemLikeContainer {
    BlockQuote,
    /// ordered list
    OL,
    /// unordered list
    UL,
    /// description list
    DL,
}

#[derive(Debug, Clone, Copy)]
pub enum GeneralItemLike {
    /// of [ItemLikeContainer::OL] or [ItemLikeContainer::UL].
    LI,
    /// of [ItemLikeContainer::DL].
    DT,
    /// of [ItemLikeContainer::DL].
    DD,
}

#[derive(Debug)]
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

#[derive(Debug)]
pub struct TopLeafParagraph {
    pub meta: Meta,

    pub new_line: Option<NewLine>,
}
impl TopLeafParagraph {
    /// 返回的事件属于 `Block` 分组。
    pub fn make_enter_event(&self) -> Event {
        ev!(Block, EnterParagraph(self.meta.id.into()))
    }

    /// 返回的事件属于 `Block` 分组。
    pub fn make_exit_event(self, line_end: LineNumber) -> Event {
        self.meta.make_exit_event(line_end)
    }
}

#[derive(Debug)]
pub struct TopLeafHeading {
    pub meta: Meta,

    pub level: usize,

    pub has_content_before: bool,
}
impl TopLeafHeading {
    /// 返回的事件属于 `Block` 分组。
    pub fn make_enter_event(&self) -> Event {
        match self.level {
            1 => ev!(Block, EnterHeading1(self.meta.id.into())),
            2 => ev!(Block, EnterHeading2(self.meta.id.into())),
            3 => ev!(Block, EnterHeading3(self.meta.id.into())),
            4 => ev!(Block, EnterHeading4(self.meta.id.into())),
            5 => ev!(Block, EnterHeading5(self.meta.id.into())),
            6 => ev!(Block, EnterHeading6(self.meta.id.into())),
            _ => unreachable!(),
        }
    }

    /// 返回的事件属于 `Block` 分组。
    pub fn make_exit_event(self, line_end: LineNumber) -> Event {
        self.meta.make_exit_event(line_end)
    }
}

#[derive(Debug)]
pub struct TopLeafCodeBlock {
    pub meta: Meta,

    pub backticks: usize,
    pub indent: usize,

    pub state: TopLeafCodeBlockState,
}
#[derive(Debug)]
pub enum TopLeafCodeBlockState {
    InInfoString,
    InCode(TopLeafCodeBlockStateInCode),
}
#[derive(Debug)]
pub enum TopLeafCodeBlockStateInCode {
    AtFirstLineBeginning,
    AtLineBeginning(NewLine),
    Normal,
}
impl TopLeafCodeBlock {
    /// 返回的事件属于 `Block` 分组。
    pub fn make_enter_event(&self) -> Event {
        ev!(Block, EnterCodeBlock(self.meta.id.into()))
    }

    /// 返回的事件属于 `Block` 分组。
    pub fn make_exit_event(self, line_end: LineNumber) -> Event {
        self.meta.make_exit_event(line_end)
    }
}
