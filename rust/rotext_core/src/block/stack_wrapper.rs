use core::ops::Range;

use crate::{
    events::{ev, Call, ExitBlock, NewLine},
    types::{BlockId, LineNumber, Stack},
    Event,
};

use super::parser_inner::ParserInnerShallowSnapshot;

pub struct StackWrapper<TStack: Stack<StackEntry>> {
    stack: TStack,
    leaf: Option<Leaf>,

    item_likes_in_stack: usize,
    tables_in_stack: usize,
    calls_in_stack: usize,

    should_reset_state: bool,
}

impl<TStack: Stack<StackEntry>> StackWrapper<TStack> {
    pub fn new() -> Self {
        Self {
            stack: TStack::new(),
            leaf: None,
            item_likes_in_stack: 0,
            tables_in_stack: 0,
            calls_in_stack: 0,
            should_reset_state: false,
        }
    }

    pub fn as_slice(&self) -> &[StackEntry] {
        self.stack.as_slice()
    }

    pub fn is_empty(&self) -> bool {
        self.leaf.is_none() && self.stack.as_slice().is_empty()
    }

    pub fn item_likes_in_stack(&self) -> usize {
        self.item_likes_in_stack
    }
    pub fn tables_in_stack(&self) -> usize {
        self.tables_in_stack
    }
    pub fn calls_in_stack(&self) -> usize {
        self.calls_in_stack
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
        if self.leaf.is_some() {
            return false;
        }
        matches!(
            self.stack.as_slice().last(),
            Some(StackEntry::ItemLikeContainer(_))
        )
    }

    pub fn top_is_description_term(&self) -> bool {
        if self.leaf.is_some() {
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
        if self.leaf.is_some() {
            return false;
        }
        matches!(self.stack.as_slice().last(), Some(StackEntry::Table(_)))
    }

    pub fn top_is_call(&self) -> bool {
        if self.leaf.is_some() {
            return false;
        }
        matches!(self.stack.as_slice().last(), Some(StackEntry::Call(_)))
    }

    /// 向栈中推入一个 item-like entry。
    ///
    /// 调用者应保证 `self.leaf` 为 `None`。
    pub fn push_item_like(&mut self, stack_entry: StackEntryItemLike) -> crate::Result<()> {
        self.try_push(stack_entry.into())?;
        Ok(())
    }

    /// 向栈中推入一个 item-like 容器 entry。
    ///
    /// 调用者应保证 `self.leaf` 为 `None`。
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
    /// 调用者应保证 `self.leaf` 为 `None`。
    pub fn push_table(&mut self, stack_entry: StackEntryTable) -> crate::Result<()> {
        self.try_push(stack_entry.into())?;
        self.tables_in_stack += 1;
        Ok(())
    }

    /// 向栈中推入一个 call entry。
    ///
    /// 调用者应保证 `self.leaf` 为 `None`。
    pub fn push_call(&mut self, stack_entry: StackEntryCall) -> crate::Result<()> {
        self.try_push(stack_entry.into())?;
        self.calls_in_stack += 1;
        Ok(())
    }

    /// 尝试向栈中推入一个 entry。
    ///
    /// 调用者应保证 `self.leaf` 为 `None`。
    fn try_push(&mut self, entry: StackEntry) -> crate::Result<()> {
        debug_assert!(self.leaf.is_none());

        self.stack.try_push(entry)
    }

    pub fn push_leaf(&mut self, entry: Leaf) {
        debug_assert!(self.leaf.is_none());

        self.leaf = Some(entry);
    }

    pub fn has_leaf(&self) -> bool {
        self.leaf.is_some()
    }

    /// 从栈中推出一个 entry。
    ///
    /// 调用者应保证 `self.leaf` 为 `None`。
    pub fn pop(&mut self) -> Option<StackEntry> {
        debug_assert!(self.leaf.is_none());

        let popped = self.stack.pop()?;

        match popped {
            StackEntry::ItemLike(_) => {}
            StackEntry::ItemLikeContainer(_) => self.item_likes_in_stack -= 1,
            StackEntry::Table(_) => self.tables_in_stack -= 1,
            StackEntry::Call(_) => self.calls_in_stack -= 1,
        }

        Some(popped)
    }

    pub fn pop_leaf(&mut self) -> Option<Leaf> {
        self.leaf.take()
    }
}

pub enum StackEntry {
    ItemLike(StackEntryItemLike),
    ItemLikeContainer(StackEntryItemLikeContainer),
    Table(StackEntryTable),
    Call(StackEntryCall),
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
impl From<StackEntryCall> for StackEntry {
    fn from(value: StackEntryCall) -> Self {
        Self::Call(value)
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

pub struct StackEntryCall {
    pub meta: Meta,
}
impl StackEntryCall {
    /// 返回的事件属于 `Block` 分组。
    pub fn make_enter_event(&self, is_extension: bool, name: Range<usize>) -> Event {
        let call = Call {
            id: self.meta.id,
            name,
        };
        if is_extension {
            ev!(Block, EnterCallOnExtension(call))
        } else {
            ev!(Block, EnterCallOnTemplate(call))
        }
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
pub enum Leaf {
    Paragraph(LeafParagraph),
    Heading(LeafHeading),
    CodeBlock(LeafCodeBlock),
    /// 正在匹配潜在的调用的名称。如果没有匹配到名称，或者名称之后并非 `||`、`??` 或 `}}`，
    /// 则不将正在解析的内容视为调用。
    PotentialCallBeginning(LeafPotentialCallBeginning),
    /// 正在匹配调用的参数可能存在的名称部分。（包含名称及 `=`。）
    CallArgumentBeginning(LeafCallArgumentBeginning),
    CallVerbatimArgumentValue(LeafCallVerbatimArgumentValue),
}
impl From<LeafParagraph> for Leaf {
    fn from(value: LeafParagraph) -> Self {
        Self::Paragraph(value)
    }
}
impl From<LeafHeading> for Leaf {
    fn from(value: LeafHeading) -> Self {
        Self::Heading(value)
    }
}
impl From<LeafCodeBlock> for Leaf {
    fn from(value: LeafCodeBlock) -> Self {
        Self::CodeBlock(value)
    }
}
impl From<LeafPotentialCallBeginning> for Leaf {
    fn from(value: LeafPotentialCallBeginning) -> Self {
        Self::PotentialCallBeginning(value)
    }
}
impl From<LeafCallArgumentBeginning> for Leaf {
    fn from(value: LeafCallArgumentBeginning) -> Self {
        Self::CallArgumentBeginning(value)
    }
}
impl From<LeafCallVerbatimArgumentValue> for Leaf {
    fn from(value: LeafCallVerbatimArgumentValue) -> Self {
        Self::CallVerbatimArgumentValue(value)
    }
}

#[derive(Debug)]
pub struct LeafParagraph {
    pub meta: Meta,

    pub new_line: Option<NewLine>,
}
impl LeafParagraph {
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
pub struct LeafHeading {
    pub meta: Meta,

    pub level: usize,

    pub has_content_before: bool,
}
impl LeafHeading {
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
pub struct LeafCodeBlock {
    pub meta: Meta,

    pub backticks: usize,
    pub indent: usize,

    pub state: LeafCodeBlockState,
}
#[derive(Debug)]
pub enum LeafCodeBlockState {
    InInfoString,
    InCode(LeafVerbatimParseState),
}
impl LeafCodeBlock {
    /// 返回的事件属于 `Block` 分组。
    pub fn make_enter_event(&self) -> Event {
        ev!(Block, EnterCodeBlock(self.meta.id.into()))
    }

    /// 返回的事件属于 `Block` 分组。
    pub fn make_exit_event(self, line_end: LineNumber) -> Event {
        self.meta.make_exit_event(line_end)
    }
}

#[derive(Debug)]
pub struct LeafPotentialCallBeginning {
    pub shallow_snapshot: ParserInnerShallowSnapshot,

    pub name_part: Option<PotentialCallBeginningNamePart>,
}
#[derive(Debug)]
pub struct PotentialCallBeginningNamePart {
    pub is_extension: bool,
    pub name: Range<usize>,
}

#[derive(Debug)]
pub struct LeafCallArgumentBeginning {
    pub shallow_snapshot: ParserInnerShallowSnapshot,

    pub name_part: Option<ParserInnerShallowSnapshotNamePart>,
}
#[derive(Debug)]
pub struct ParserInnerShallowSnapshotNamePart {
    /// 如果为真，则代表参数的值是逐字内容；否则，参数的值是块级元素序列。
    pub is_verbatim: bool,
    pub name: Range<usize>,
}

#[derive(Debug)]
pub struct LeafCallVerbatimArgumentValue {
    pub state: LeafVerbatimParseState,
}

#[derive(Debug)]
pub enum LeafVerbatimParseState {
    AtFirstLineBeginning,
    AtLineBeginning(NewLine),
    Normal,
}
