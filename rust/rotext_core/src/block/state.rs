use crate::{types::Stack, Event};

use super::{
    parser_inner::ParserInnerShallowSnapshot,
    stack_wrapper::{StackEntryItemLike, StackEntryItemLikeContainer, StackWrapper},
    StackEntry,
};

pub enum State {
    Expecting(Expecting),
    /// 持续从栈中推出内容并产出对应的退出事件，直到满足特定条件，在那之后执行要做的事情。
    Exiting(Exiting),
    Ended,
    ToApplyShallowSnapshot(
        /// Option 仅用于处理所有权，`None` 为无效状态。
        Option<ToApplyShallowSnapshot>,
    ),
}
impl From<Expecting> for State {
    fn from(value: Expecting) -> Self {
        Self::Expecting(value)
    }
}
impl From<Exiting> for State {
    fn from(value: Exiting) -> Self {
        Self::Exiting(value)
    }
}
impl From<ToApplyShallowSnapshot> for State {
    fn from(value: ToApplyShallowSnapshot) -> Self {
        Self::ToApplyShallowSnapshot(Some(value))
    }
}

#[derive(Clone, Copy)]
pub enum Expecting {
    ItemLikeOpening,
    BracedOpening,
    LeafContent,
}

pub struct Exiting {
    pub until: ExitingUntil,
    /// 完成退出后要做什么。
    ///
    /// XXX: 必定为 `Some`，因为在它被 `take` 走后 parser 的状态必定被新的状态覆盖。这里
    /// 使用 Option 仅仅是为了 workaround rust 的生命周期。
    pub and_then: Option<ExitingAndThen>,
}
pub enum ExitingUntil {
    OnlyNItemLikesRemain {
        n: usize,
        should_also_exit_containee_in_last_container: bool,
    },
    TopIsTable {
        should_also_exit_table: bool,
    },
    TopIsCall {
        should_also_exit_call: bool,
    },
    TopIsAwareOfDoublePipes,
    StackIsEmpty,
}
pub enum ExitingAndThen {
    EnterItemLikeAndExpectItemLike {
        container: Option<StackEntryItemLikeContainer>,
        item_like: StackEntryItemLike,
    },
    ExpectBracedOpening,
    /// 包含属于 `Block` 分组的事件。
    YieldAndExpectBracedOpening(Event),
    PushTopLeafCallArgumentBeginningAndExpectBracedOpening,
    End,
    ToBeDetermined,
}
impl Exiting {
    pub fn new(until: ExitingUntil, and_then: ExitingAndThen) -> Self {
        Self {
            until,
            and_then: Some(and_then),
        }
    }
}

pub struct ToApplyShallowSnapshot {
    pub shallow_snapshot: ParserInnerShallowSnapshot,
    pub and_then: ToApplyShallowSnapshotAndThen,
}
pub enum ToApplyShallowSnapshotAndThen {
    TryParseAsParagraph,
    YieldAndEnterCallVerbatimArgumentValue(Event),
    YieldAndExpectBracedOpening(Event),
}

#[derive(Clone)]
pub enum ItemLikesState {
    MatchingLastLine(ItemLikesStateMatchingLastLine),
    ProcessingNew,
}
impl From<ItemLikesStateMatchingLastLine> for ItemLikesState {
    fn from(value: ItemLikesStateMatchingLastLine) -> Self {
        Self::MatchingLastLine(value)
    }
}
impl ItemLikesState {
    pub fn has_unprocessed_item_likes_at_current_line(&self) -> bool {
        matches!(&self, ItemLikesState::MatchingLastLine(_))
    }
}

#[derive(Clone)]
pub struct ItemLikesStateMatchingLastLine {
    /// 根据上一行的情况匹配当前行的 item-likes。
    last_total: usize,
    /// 处理当前行新的 item-likes。
    current_last_accessed: LastAccessedItemLikeAtCurrentLine,
}
impl ItemLikesStateMatchingLastLine {
    pub fn new(last_total: usize) -> Self {
        debug_assert!(last_total > 0);
        Self {
            last_total,
            current_last_accessed: LastAccessedItemLikeAtCurrentLine::new(),
        }
    }

    pub fn processed_item_likes(&self) -> usize {
        self.current_last_accessed.nth
    }

    pub fn first_unprocessed_item_like<'a, TStack: Stack<StackEntry>>(
        &mut self,
        stack: &'a StackWrapper<TStack>,
    ) -> &'a StackEntryItemLikeContainer {
        self.update_last_accessed_item_like_next_index(stack)
    }

    pub fn mark_first_unprocessed_item_like_as_processed_at_current_line<
        TStack: Stack<StackEntry>,
    >(
        &mut self,
        stack: &StackWrapper<TStack>,
    ) -> bool {
        self.current_last_accessed.nth += 1;
        self.update_last_accessed_item_like_next_index(stack);
        self.current_last_accessed.next_index += 1;

        self.processed_item_likes() == self.last_total
    }

    fn update_last_accessed_item_like_next_index<'a, TStack: Stack<StackEntry>>(
        &mut self,
        stack: &'a StackWrapper<TStack>,
    ) -> &'a StackEntryItemLikeContainer {
        let slice = stack.as_slice();
        loop {
            match &slice[self.current_last_accessed.next_index] {
                StackEntry::ItemLikeContainer(stack_entry) => break stack_entry,
                _ => self.current_last_accessed.next_index += 1,
            }
        }
    }
}

#[derive(Clone)]
pub struct LastAccessedItemLikeAtCurrentLine {
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
