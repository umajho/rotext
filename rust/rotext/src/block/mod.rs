mod line;
mod parser_inner;
mod stack_wrapper;
mod state;
mod types;
mod utils;

#[cfg(test)]
mod test_support;
#[cfg(test)]
mod tests;

pub use stack_wrapper::StackEntry;

use crate::{
    common::m,
    events::{BlockEvent, ThematicBreak},
    utils::stack::Stack,
};

use state::{
    Exiting, ExitingAndThen, ExitingUntil, Expecting, ItemLikesState,
    ItemLikesStateMatchingLastLine, State,
};

use parser_inner::ParserInner;
use stack_wrapper::{
    GeneralItemLike, ItemLikeContainer, Meta, StackEntryItemLike, StackEntryItemLikeContainer,
    StackEntryTable, TopLeaf, TopLeafCodeBlock, TopLeafHeading, TopLeafParagraph,
};
use types::{cast_tym, CursorContext, Tym, YieldContext, TYM_UNIT};
use utils::count_continuous_character;

pub struct Parser<'a, TStack: Stack<StackEntry>> {
    input: &'a [u8],
    state: State,
    inner: ParserInner<TStack>,
    /// 其实只有在 `state` 为 [Expecting::ItemLikeOpening] 时有效，但将
    /// [ItemLikesState] 作为 [Expecting::ItemLikeOpening] 的字段并不是很可行。之后有
    /// 函数既需要传入 `&mut state`，又需要知道 `expecting` 的值是什么，而现在的
    /// [Expecting] 各分支都没有字段，[Copy] 起来很便宜。如果将
    /// [ItemLikesState] 作为 [Expecting::ItemLikeOpening] 的字段，开销一下子上来了。
    item_likes_state: ItemLikesState,

    #[cfg(debug_assertions)]
    is_errored: bool,
}

impl<'a, TStack: Stack<StackEntry>> Parser<'a, TStack> {
    pub fn new(input: &'a [u8]) -> Self {
        Self {
            input,
            state: Expecting::ItemLikeOpening.into(),
            inner: ParserInner::new(),
            item_likes_state: ItemLikesState::ProcessingNew,

            #[cfg(debug_assertions)]
            is_errored: false,
        }
    }

    pub fn next(&mut self) -> Option<crate::Result<BlockEvent>> {
        #[cfg(debug_assertions)]
        {
            assert!(!self.is_errored);
        }
        debug_assert!(!matches!(self.state, State::Ended));

        loop {
            if let Some(ev) = self.inner.pop_to_be_yielded() {
                break Some(Ok(ev));
            }

            let result: crate::Result<Tym<3>> = match &mut self.state {
                State::Exiting(exiting_branches) => match Self::exit(
                    &mut self.inner,
                    &mut self.item_likes_state,
                    exiting_branches,
                ) {
                    Ok((tym, state)) => {
                        if let Some(state) = state {
                            self.state = state;
                        }
                        Ok(tym)
                    }
                    Err(err) => Err(err),
                }
                .map(|tym| cast_tym!(tym)),
                State::Ended => {
                    break None;
                }
                State::Expecting(expecting) => {
                    if self.inner.stack.should_reset_state() {
                        self.inner.stack.reset_should_reset_state();
                        let item_likes_in_stack_at_last_line =
                            self.inner.stack.item_likes_in_stack();
                        *expecting = Expecting::ItemLikeOpening;
                        self.item_likes_state = if item_likes_in_stack_at_last_line > 0 {
                            ItemLikesStateMatchingLastLine::new(item_likes_in_stack_at_last_line)
                                .into()
                        } else {
                            ItemLikesState::ProcessingNew
                        };
                    }
                    self.inner.reset_current_expecting();

                    let expecting = *expecting;
                    self.parse(expecting)
                }
            };
            match result {
                Ok(tym) => self.inner.enforce_to_yield_mark(tym),
                Err(err) => {
                    #[cfg(debug_assertions)]
                    {
                        self.is_errored = true;
                    }
                    break Some(Err(err));
                }
            }
        }
    }

    #[inline(always)]
    fn parse(&mut self, mut expecting: Expecting) -> crate::Result<Tym<3>> {
        let spaces = count_continuous_character(self.input, b' ', self.inner.cursor());
        if spaces > 0 {
            self.inner.move_cursor_forward(spaces);
            self.inner.current_expecting.set_spaces_before(spaces);
        }

        let Some(&first_char) = self.input.get(self.inner.cursor()) else {
            self.state = if self.inner.stack.is_empty() {
                State::Ended
            } else {
                Exiting::new(ExitingUntil::StackIsEmpty, ExitingAndThen::End).into()
            };

            return Ok(TYM_UNIT.into());
        };

        loop {
            match expecting {
                Expecting::ItemLikeOpening => {
                    if !self
                        .item_likes_state
                        .has_unprocessed_item_likes_at_current_line()
                        && self.inner.stack.is_top_leaf_some()
                    {
                        expecting = Expecting::LeafContent;
                        self.state = expecting.into();
                        continue;
                    }
                    break branch::item_like::parse_opening_and_process(
                        self.input,
                        &mut self.state,
                        &mut self.inner,
                        &mut self.item_likes_state,
                        first_char,
                    )
                    .map(|tym| cast_tym!(tym));
                }
                Expecting::BracedOpening => {
                    if self.inner.stack.is_top_leaf_some() {
                        expecting = Expecting::LeafContent;
                        self.state = expecting.into();
                        continue;
                    }
                    break branch::braced::parse_opening_and_process(
                        self.input,
                        &mut self.state,
                        &mut self.inner,
                        first_char,
                    )
                    .map(|tym| cast_tym!(tym));
                }
                Expecting::LeafContent => {
                    if let Some(top_leaf) = self.inner.stack.pop_top_leaf() {
                        break leaf::parse_content_and_process(
                            self.input,
                            &mut self.state,
                            &mut self.inner,
                            top_leaf,
                        )
                        .map(|tym| cast_tym!(tym));
                    }
                    break leaf::parse_opening_and_process(
                        self.input,
                        &mut self.state,
                        &mut self.inner,
                        first_char,
                    )
                    .map(|tym| cast_tym!(tym));
                }
            }
        }
    }

    fn exit(
        inner: &mut ParserInner<TStack>,
        item_likes_state: &mut ItemLikesState,
        exiting: &mut Exiting,
    ) -> crate::Result<(Tym<3>, Option<State>)> {
        if let Some(top_leaf) = inner.stack.pop_top_leaf() {
            let tym: Tym<2> = match top_leaf {
                TopLeaf::Paragraph(top_leaf) => leaf::paragraph::exit(inner, top_leaf).into(),
                TopLeaf::Heading(top_leaf) => leaf::heading::exit(inner, top_leaf).into(),
                TopLeaf::CodeBlock(top_leaf) => leaf::code_block::exit(inner, top_leaf),
            };
            return Ok((cast_tym!(tym), None));
        }

        let mut to_be_yielded_based_on_context: Option<BlockEvent> = None;

        let (is_done, should_exit_top) = match exiting.until {
            ExitingUntil::OnlyNItemLikesRemain {
                n,
                should_also_exit_containee_in_last_container,
            } => {
                debug_assert!(inner.stack.item_likes_in_stack() >= n);
                let mut is_done = inner.stack.item_likes_in_stack() == n;
                if should_also_exit_containee_in_last_container {
                    is_done = is_done && inner.stack.top_is_item_like_container();
                }
                (is_done, !is_done)
            }
            ExitingUntil::TopIsTable {
                should_also_exit_table,
            } => (inner.stack.top_is_table(), should_also_exit_table),
            ExitingUntil::TopIsAwareOfDoublePipes => {
                if inner.stack.top_is_table() {
                    to_be_yielded_based_on_context = Some(BlockEvent::IndicateTableDataCell);
                    (true, false)
                } else {
                    (false, true)
                }
            }
            ExitingUntil::StackIsEmpty => {
                let is_done = inner.stack.is_empty();
                (is_done, !is_done)
            }
        };

        let tym_a = if should_exit_top {
            match inner.stack.pop().unwrap() {
                StackEntry::ItemLike(stack_entry) => branch::item_like::exit(inner, stack_entry)?,
                StackEntry::ItemLikeContainer(stack_entry) => {
                    branch::item_like::exit_container(inner, stack_entry)?
                }
                StackEntry::Table(stack_entry) => branch::braced::table::exit(inner, stack_entry)?,
            }
        } else {
            TYM_UNIT.into()
        };

        let (tym_b, new_state) = if is_done {
            debug_assert!(exiting.and_then.is_some());
            let and_then = unsafe { exiting.and_then.take().unwrap_unchecked() };
            #[cfg(debug_assertions)]
            {
                if !matches!(
                    and_then,
                    ExitingAndThen::YieldBasedOnContextAndExpectBracedOpening
                ) {
                    assert!(to_be_yielded_based_on_context.is_none());
                }
            }
            match and_then {
                ExitingAndThen::EnterItemLikeAndExpectItemLike {
                    container,
                    item_like,
                } => {
                    let tym_a = if let Some(container) = container {
                        let ev = container.make_enter_event();
                        inner.stack.push_item_like_container(container)?;
                        inner.r#yield(ev)
                    } else {
                        TYM_UNIT.into()
                    };
                    let tym_b = {
                        let ev = item_like.make_enter_event();
                        inner.stack.push_item_like(item_like)?;
                        inner.r#yield(ev)
                    };
                    *item_likes_state = ItemLikesState::ProcessingNew;
                    (tym_a.add(tym_b), Some(Expecting::ItemLikeOpening.into()))
                }
                ExitingAndThen::ExpectBracedOpening => {
                    (TYM_UNIT.into(), Some(Expecting::BracedOpening.into()))
                }
                ExitingAndThen::YieldAndExpectBracedOpening(ev) => (
                    inner.r#yield(ev).into(),
                    Some(Expecting::BracedOpening.into()),
                ),
                ExitingAndThen::YieldBasedOnContextAndExpectBracedOpening => {
                    debug_assert!(to_be_yielded_based_on_context.is_some());
                    (
                        inner
                            .r#yield(unsafe { to_be_yielded_based_on_context.unwrap_unchecked() })
                            .into(),
                        Some(Expecting::BracedOpening.into()),
                    )
                }
                ExitingAndThen::End => (TYM_UNIT.into(), Some(State::Ended)),
            }
        } else {
            debug_assert!(to_be_yielded_based_on_context.is_none());
            (TYM_UNIT.into(), None)
        };

        Ok((tym_a.add(tym_b), new_state))
    }
}

impl<'a, TStack: Stack<StackEntry>> Iterator for Parser<'a, TStack> {
    type Item = crate::Result<BlockEvent>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next()
    }
}

mod branch {
    use super::*;

    pub mod item_like {
        use super::*;

        pub fn parse_opening_and_process<TStack: Stack<StackEntry>>(
            input: &[u8],
            state: &mut State,
            inner: &mut ParserInner<TStack>,
            item_likes_state: &mut ItemLikesState,
            first_char: u8,
        ) -> crate::Result<Tym<3>> {
            use GeneralItemLike as I;
            use ItemLikeContainer as G;

            match first_char {
                m!('>') if is_indeed_opening_and_consume_if_true(input, inner) => {
                    process_greater_than_opening(inner, item_likes_state).map(|tym| cast_tym!(tym))
                }
                m!('#') if is_indeed_opening_and_consume_if_true(input, inner) => {
                    process_general_opening(state, inner, item_likes_state, G::OL, I::LI)
                        .map(|tym| cast_tym!(tym))
                }
                m!('*') if is_indeed_opening_and_consume_if_true(input, inner) => {
                    process_general_opening(state, inner, item_likes_state, G::UL, I::LI)
                        .map(|tym| cast_tym!(tym))
                }
                m!(';') if is_indeed_opening_and_consume_if_true(input, inner) => {
                    process_general_opening(state, inner, item_likes_state, G::DL, I::DT)
                        .map(|tym| cast_tym!(tym))
                }
                m!(':') if is_indeed_opening_and_consume_if_true(input, inner) => {
                    process_general_opening(state, inner, item_likes_state, G::DL, I::DD)
                        .map(|tym| cast_tym!(tym))
                }
                _ => match item_likes_state {
                    ItemLikesState::MatchingLastLine(matching_last_line) => {
                        *state = Exiting::new(
                            ExitingUntil::OnlyNItemLikesRemain {
                                n: matching_last_line.processed_item_likes(),
                                should_also_exit_containee_in_last_container: false,
                            },
                            ExitingAndThen::ExpectBracedOpening,
                        )
                        .into();
                        Ok(TYM_UNIT.into())
                    }
                    ItemLikesState::ProcessingNew => {
                        *state = State::Expecting(Expecting::BracedOpening);
                        braced::parse_opening_and_process(input, state, inner, first_char)
                            .map(|tym| cast_tym!(tym))
                    }
                },
            }
        }

        fn process_greater_than_opening<TStack: Stack<StackEntry>>(
            inner: &mut ParserInner<TStack>,
            item_likes_state: &mut ItemLikesState,
        ) -> crate::Result<Tym<1>> {
            let tym = match item_likes_state {
                ItemLikesState::MatchingLastLine(matching_last_line) => {
                    let is_all_processed = matching_last_line
                        .mark_first_unprocessed_item_like_as_processed_at_current_line(
                            &inner.stack,
                        );
                    if is_all_processed {
                        *item_likes_state = ItemLikesState::ProcessingNew;
                    }
                    TYM_UNIT.into()
                }
                ItemLikesState::ProcessingNew => {
                    let id = inner.pop_block_id();
                    inner
                        .stack
                        .push_item_like_container(StackEntryItemLikeContainer {
                            meta: Meta::new(id, inner.current_line()),
                            r#type: ItemLikeContainer::BlockQuote,
                        })?;
                    inner.r#yield(BlockEvent::EnterBlockQuote(id.into()))
                }
            };

            Ok(tym)
        }

        fn process_general_opening<TStack: Stack<StackEntry>>(
            state: &mut State,
            inner: &mut ParserInner<TStack>,
            item_likes_state: &mut ItemLikesState,
            container: ItemLikeContainer,
            item_like: GeneralItemLike,
        ) -> crate::Result<Tym<2>> {
            let tym = match item_likes_state {
                ItemLikesState::MatchingLastLine(matching_last_line) => {
                    let stack_entry = matching_last_line.first_unprocessed_item_like(&inner.stack);
                    if stack_entry.r#type == container {
                        matching_last_line
                            .mark_first_unprocessed_item_like_as_processed_at_current_line(
                                &inner.stack,
                            );
                        *state = Exiting::new(
                            ExitingUntil::OnlyNItemLikesRemain {
                                n: matching_last_line.processed_item_likes(),
                                should_also_exit_containee_in_last_container: true,
                            },
                            ExitingAndThen::EnterItemLikeAndExpectItemLike {
                                container: None,
                                item_like: make_stack_entry_from_general_item_like(
                                    item_like, inner,
                                ),
                            },
                        )
                        .into()
                    } else {
                        *state = Exiting::new(
                            ExitingUntil::OnlyNItemLikesRemain {
                                n: matching_last_line.processed_item_likes(),
                                should_also_exit_containee_in_last_container: false,
                            },
                            ExitingAndThen::EnterItemLikeAndExpectItemLike {
                                container: Some(make_stack_entry_from_item_like_container(
                                    container, inner,
                                )),
                                item_like: make_stack_entry_from_general_item_like(
                                    item_like, inner,
                                ),
                            },
                        )
                        .into();
                    }
                    TYM_UNIT.into()
                }
                ItemLikesState::ProcessingNew => {
                    let tym_a = {
                        let stack_entry =
                            make_stack_entry_from_item_like_container(container, inner);
                        let ev = stack_entry.make_enter_event();
                        inner.stack.push_item_like_container(stack_entry)?;
                        inner.r#yield(ev)
                    };
                    let tym_b = {
                        let stack_entry = make_stack_entry_from_general_item_like(item_like, inner);
                        let ev = stack_entry.make_enter_event();
                        inner.stack.push_item_like(stack_entry)?;
                        inner.r#yield(ev)
                    };

                    tym_a.add(tym_b)
                }
            };

            Ok(tym)
        }

        fn is_indeed_opening_and_consume_if_true<TStack: Stack<StackEntry>>(
            input: &[u8],
            inner: &mut ParserInner<TStack>,
        ) -> bool {
            match input.get(inner.cursor() + 1) {
                Some(b' ') => inner.move_cursor_forward(1 + " ".len()),
                None | Some(b'\r' | b'\n') => inner.move_cursor_forward(1),
                // TODO: 也许可以一步到位调用 `leaf::paragraph::enter_if_not_blank(input, inner, 1)`。
                _ => return false,
            }

            true
        }

        fn make_stack_entry_from_general_item_like<TStack: Stack<StackEntry>>(
            item_like: GeneralItemLike,
            inner: &mut ParserInner<TStack>,
        ) -> StackEntryItemLike {
            StackEntryItemLike {
                meta: Meta::new(inner.pop_block_id(), inner.current_line()),
                r#type: item_like,
            }
        }

        fn make_stack_entry_from_item_like_container<TStack: Stack<StackEntry>>(
            item_like: ItemLikeContainer,
            inner: &mut ParserInner<TStack>,
        ) -> StackEntryItemLikeContainer {
            StackEntryItemLikeContainer {
                meta: Meta::new(inner.pop_block_id(), inner.current_line()),
                r#type: item_like,
            }
        }

        pub fn exit_container<TStack: Stack<StackEntry>>(
            inner: &mut ParserInner<TStack>,
            stack_entry: StackEntryItemLikeContainer,
        ) -> crate::Result<Tym<1>> {
            let tym = inner.r#yield(stack_entry.make_exit_event(inner.current_line()));

            Ok(tym)
        }

        pub fn exit<TStack: Stack<StackEntry>>(
            inner: &mut ParserInner<TStack>,
            stack_entry: StackEntryItemLike,
        ) -> crate::Result<Tym<1>> {
            let tym = inner.r#yield(stack_entry.make_exit_event(inner.current_line()));

            Ok(tym)
        }
    }

    pub mod braced {
        use super::*;

        pub fn parse_opening_and_process<TStack: Stack<StackEntry>>(
            input: &[u8],
            state: &mut State,
            inner: &mut ParserInner<TStack>,
            first_char: u8,
        ) -> crate::Result<Tym<3>> {
            match first_char {
                m!('{') => match input.get(inner.cursor() + 1) {
                    Some(m!('|')) => {
                        inner.move_cursor_forward("{|".len());
                        table::enter(state, inner).map(|tym| cast_tym!(tym))
                    }
                    _ => leaf::paragraph::enter_if_not_blank(input, state, inner, 1)
                        .map(|tym| cast_tym!(tym)),
                },
                _ => leaf::parse_opening_and_process(input, state, inner, first_char)
                    .map(|tym| cast_tym!(tym)),
            }
        }

        pub mod table {
            use super::*;

            pub(super) fn enter<TStack: Stack<StackEntry>>(
                state: &mut State,
                inner: &mut ParserInner<TStack>,
            ) -> crate::Result<Tym<1>> {
                *state = Expecting::BracedOpening.into();

                let id = inner.pop_block_id();
                let stack_entry = StackEntryTable {
                    meta: Meta::new(id, inner.current_line()),
                };
                let ev = stack_entry.make_enter_event();
                inner.stack.push_table(stack_entry)?;
                let tym = inner.r#yield(ev);

                Ok(tym)
            }

            #[derive(Debug, PartialEq, Eq)]
            pub enum TableRelatedEnd {
                TableClosing,
                TableCaptionIndicator,
                TableRowIndicator,
                TableHeaderCellIndicator,
                DoublePipes,
            }
            impl TableRelatedEnd {
                pub fn process(self, state: &mut State) -> Tym<0> {
                    *state = match self {
                        TableRelatedEnd::TableClosing => Exiting::new(
                            ExitingUntil::TopIsTable {
                                should_also_exit_table: true,
                            },
                            ExitingAndThen::ExpectBracedOpening,
                        )
                        .into(),
                        TableRelatedEnd::TableCaptionIndicator => Exiting::new(
                            ExitingUntil::TopIsTable {
                                should_also_exit_table: false,
                            },
                            ExitingAndThen::YieldAndExpectBracedOpening(
                                BlockEvent::IndicateTableCaption,
                            ),
                        )
                        .into(),
                        TableRelatedEnd::TableRowIndicator => Exiting::new(
                            ExitingUntil::TopIsTable {
                                should_also_exit_table: false,
                            },
                            ExitingAndThen::YieldAndExpectBracedOpening(
                                BlockEvent::IndicateTableRow,
                            ),
                        )
                        .into(),
                        TableRelatedEnd::TableHeaderCellIndicator => Exiting::new(
                            ExitingUntil::TopIsTable {
                                should_also_exit_table: false,
                            },
                            ExitingAndThen::YieldAndExpectBracedOpening(
                                BlockEvent::IndicateTableHeaderCell,
                            ),
                        )
                        .into(),
                        TableRelatedEnd::DoublePipes => Exiting::new(
                            ExitingUntil::TopIsAwareOfDoublePipes,
                            ExitingAndThen::YieldBasedOnContextAndExpectBracedOpening,
                        )
                        .into(),
                    };

                    cast_tym!(TYM_UNIT)
                }
            }

            pub fn parse_end<TCtx: CursorContext>(
                input: &[u8],
                ctx: &mut TCtx,
                first_char: u8,
                is_caption_applicable: bool,
            ) -> Option<TableRelatedEnd> {
                let &second_char = input.get(ctx.cursor() + 1)?;
                let end = match first_char {
                    m!('|') => match second_char {
                        m!('}') => TableRelatedEnd::TableClosing,
                        m!('+') if is_caption_applicable => TableRelatedEnd::TableCaptionIndicator,
                        m!('-') => TableRelatedEnd::TableRowIndicator,
                        m!('|') => TableRelatedEnd::DoublePipes,
                        _ => return None,
                    },
                    m!('!') => match second_char {
                        m!('!') => TableRelatedEnd::TableHeaderCellIndicator,
                        _ => return None,
                    },
                    _ => return None,
                };
                ctx.move_cursor_forward(2);
                Some(end)
            }

            pub fn exit<TStack: Stack<StackEntry>>(
                inner: &mut ParserInner<TStack>,
                stack_entry: StackEntryTable,
            ) -> crate::Result<Tym<1>> {
                let tym = inner.r#yield(stack_entry.make_exit_event(inner.current_line()));

                Ok(tym)
            }

            pub fn make_table_related_end_condition<TStack: Stack<StackEntry>>(
                inner: &ParserInner<TStack>,
                is_caption_applicable: bool,
            ) -> Option<line::normal::TableRelated> {
                if inner.stack.tables_in_stack() > 0 {
                    Some(line::normal::TableRelated {
                        is_caption_applicable,
                    })
                } else {
                    None
                }
            }
        }
    }
}

mod leaf {
    use super::*;

    pub fn parse_opening_and_process<TStack: Stack<StackEntry>>(
        input: &[u8],
        state: &mut State,
        inner: &mut ParserInner<TStack>,
        first_char: u8,
    ) -> crate::Result<Tym<3>> {
        match first_char {
            m!('-') => {
                let count = 1 + count_continuous_character(input, m!('-'), inner.cursor() + 1);
                if count >= 3 {
                    inner.move_cursor_forward(count);
                    thematic_break::process(inner).map(|tym| cast_tym!(tym))
                } else {
                    paragraph::enter_if_not_blank(input, state, inner, count)
                        .map(|tym| cast_tym!(tym))
                }
            }
            m!('=') => {
                let count = 1 + count_continuous_character(input, m!('='), inner.cursor() + 1);
                if (1..=6).contains(&count) && input.get(inner.cursor() + count) == Some(&b' ') {
                    inner.move_cursor_forward(count + " ".len());
                    heading::enter(inner, count).map(|tym| cast_tym!(tym))
                } else {
                    paragraph::enter_if_not_blank(input, state, inner, count)
                        .map(|tym| cast_tym!(tym))
                }
            }
            m!('`') => {
                let count = 1 + count_continuous_character(input, m!('`'), inner.cursor() + 1);
                if count >= 3 {
                    inner.move_cursor_forward(count);
                    code_block::enter(inner, count).map(|tym| cast_tym!(tym))
                } else {
                    paragraph::enter_if_not_blank(input, state, inner, count)
                        .map(|tym| cast_tym!(tym))
                }
            }
            _ => paragraph::enter_if_not_blank(input, state, inner, 0).map(|tym| cast_tym!(tym)),
        }
    }

    pub fn parse_content_and_process<TStack: Stack<StackEntry>>(
        input: &[u8],
        state: &mut State,
        inner: &mut ParserInner<TStack>,
        top_leaf: TopLeaf,
    ) -> crate::Result<Tym<3>> {
        match top_leaf {
            TopLeaf::Paragraph(top_leaf) => {
                leaf::paragraph::parse_content_and_process(input, state, inner, top_leaf)
                    .map(|tym| cast_tym!(tym))
            }
            TopLeaf::Heading(top_leaf) => {
                leaf::heading::parse_content_and_process(input, state, inner, top_leaf)
                    .map(|tym| cast_tym!(tym))
            }
            TopLeaf::CodeBlock(top_leaf) => {
                leaf::code_block::parse_content_and_process(input, inner, top_leaf)
                    .map(|tym| cast_tym!(tym))
            }
        }
    }

    mod thematic_break {
        use super::*;

        pub fn process<TStack: Stack<StackEntry>>(
            inner: &mut ParserInner<TStack>,
        ) -> crate::Result<Tym<1>> {
            let id = inner.pop_block_id();
            let tym = inner.r#yield(BlockEvent::ThematicBreak(ThematicBreak {
                id,
                line: inner.current_line(),
            }));

            Ok(tym)
        }
    }

    pub mod heading {
        use line::normal::AtxClosing;

        use super::*;

        pub(super) fn enter<TStack: Stack<StackEntry>>(
            inner: &mut ParserInner<TStack>,
            level: usize,
        ) -> crate::Result<Tym<1>> {
            let id = inner.pop_block_id();
            let top_leaf = TopLeafHeading {
                meta: Meta::new(id, inner.current_line()),
                level,
                has_content_before: false,
            };
            let ev = top_leaf.make_enter_event();
            inner.stack.push_top_leaf(top_leaf.into());
            let tym = inner.r#yield(ev);

            Ok(tym)
        }

        pub fn parse_content_and_process<TStack: Stack<StackEntry>>(
            input: &[u8],
            state: &mut State,
            inner: &mut ParserInner<TStack>,
            mut top_leaf: TopLeafHeading,
        ) -> crate::Result<Tym<2>> {
            let (mut content, end) = line::normal::parse(
                input,
                inner,
                line::normal::EndCondition {
                    on_atx_closing: Some(AtxClosing {
                        character: m!('='),
                        count: top_leaf.level,
                    }),
                    on_table_related: branch::braced::table::make_table_related_end_condition(
                        inner, false,
                    ),
                },
                if inner.current_expecting.spaces_before() > 0 {
                    line::normal::ContentBefore::Space
                } else {
                    line::normal::ContentBefore::NotSpace(0)
                },
            );

            if top_leaf.has_content_before
                && inner.current_expecting.spaces_before() > 0
                && end.is_verbatim_escaping()
            {
                content.start -= inner.current_expecting.spaces_before();
            }

            let tym_a = if !content.is_empty() {
                inner.r#yield(BlockEvent::Unparsed(content))
            } else {
                TYM_UNIT.into()
            };

            let tym_b = match end {
                line::normal::End::Eof | line::normal::End::NewLine(_) => exit(inner, top_leaf),
                line::normal::End::VerbatimEscaping(verbatim_escaping) => {
                    top_leaf.has_content_before = true;
                    inner.stack.push_top_leaf(top_leaf.into());
                    line::global_phase::process_verbatim_escaping(inner, verbatim_escaping)
                }
                line::normal::End::TableRelated(table_related_end) => {
                    let tym_a = exit(inner, top_leaf);
                    let tym_b = table_related_end.process(state);

                    tym_a.add(tym_b)
                }
                line::normal::End::None => TYM_UNIT.into(),
            };

            Ok(tym_a.add(tym_b))
        }

        pub fn exit<TStack: Stack<StackEntry>>(
            inner: &mut ParserInner<TStack>,
            top_leaf: TopLeafHeading,
        ) -> Tym<1> {
            inner.r#yield(top_leaf.make_exit_event(inner.current_line()))
        }
    }

    pub mod code_block {
        use stack_wrapper::{TopLeafCodeBlockState, TopLeafCodeBlockStateInCode};

        use super::*;

        pub(super) fn enter<TStack: Stack<StackEntry>>(
            inner: &mut ParserInner<TStack>,
            backticks: usize,
        ) -> crate::Result<Tym<1>> {
            let id = inner.pop_block_id();
            let top_leaf = TopLeafCodeBlock {
                meta: Meta::new(id, inner.current_line()),
                backticks,
                indent: inner.current_expecting.spaces_before(),
                state: TopLeafCodeBlockState::InInfoString,
            };
            let ev = top_leaf.make_enter_event();
            inner.stack.push_top_leaf(top_leaf.into());
            let tym = inner.r#yield(ev);

            Ok(tym)
        }

        pub fn parse_content_and_process<TStack: Stack<StackEntry>>(
            input: &[u8],
            inner: &mut ParserInner<TStack>,
            mut top_leaf: TopLeafCodeBlock,
        ) -> crate::Result<Tym<3>> {
            let tym = match top_leaf.state {
                TopLeafCodeBlockState::InInfoString => {
                    let (content, end) = line::verbatim::parse(
                        input,
                        inner,
                        line::verbatim::EndCondition { on_fence: None },
                        inner.current_expecting.spaces_before(),
                        None,
                    );

                    let tym_a = if !content.is_empty() {
                        inner.r#yield(BlockEvent::Text(content))
                    } else {
                        TYM_UNIT.into()
                    };

                    let tym_b = match end {
                        line::verbatim::End::Eof => TYM_UNIT.into(),
                        line::verbatim::End::NewLine(_new_line) => {
                            top_leaf.state = TopLeafCodeBlockState::InCode(
                                TopLeafCodeBlockStateInCode::AtFirstLineBeginning,
                            );
                            inner.r#yield(BlockEvent::IndicateCodeBlockCode)
                        }
                        line::verbatim::End::VerbatimEscaping(verbatim_escaping) => {
                            line::global_phase::process_verbatim_escaping(inner, verbatim_escaping)
                        }
                        line::verbatim::End::Fence => unreachable!(),
                        line::verbatim::End::None => TYM_UNIT.into(),
                    };

                    inner.stack.push_top_leaf(top_leaf.into());

                    tym_a.add(tym_b).into()
                }
                TopLeafCodeBlockState::InCode(ref in_code) => {
                    let (at_line_beginning, new_line) = match in_code {
                        TopLeafCodeBlockStateInCode::AtFirstLineBeginning => (
                            Some(line::verbatim::AtLineBeginning {
                                indent: top_leaf.indent,
                            }),
                            None,
                        ),
                        TopLeafCodeBlockStateInCode::AtLineBeginning(new_line) => (
                            Some(line::verbatim::AtLineBeginning {
                                indent: top_leaf.indent,
                            }),
                            Some(new_line.clone()),
                        ),
                        TopLeafCodeBlockStateInCode::Normal => (None, None),
                    };

                    let (content, end) = line::verbatim::parse(
                        input,
                        inner,
                        line::verbatim::EndCondition {
                            on_fence: Some(line::verbatim::Fence {
                                character: m!('`'),
                                minimum_count: top_leaf.backticks,
                            }),
                        },
                        inner.current_expecting.spaces_before(),
                        at_line_beginning,
                    );

                    let tym_a = if let Some(new_line) = new_line {
                        if !matches!(end, line::verbatim::End::Eof) {
                            inner.r#yield(BlockEvent::NewLine(new_line))
                        } else {
                            TYM_UNIT.into()
                        }
                    } else {
                        TYM_UNIT.into()
                    };

                    let tym_b = if !content.is_empty() {
                        inner.r#yield(BlockEvent::Text(content))
                    } else {
                        TYM_UNIT.into()
                    };

                    let tym_c = match end {
                        line::verbatim::End::Eof => {
                            inner.stack.push_top_leaf(top_leaf.into());
                            TYM_UNIT.into()
                        }
                        line::verbatim::End::NewLine(new_line) => {
                            top_leaf.state = TopLeafCodeBlockState::InCode(
                                TopLeafCodeBlockStateInCode::AtLineBeginning(new_line),
                            );
                            inner.stack.push_top_leaf(top_leaf.into());
                            TYM_UNIT.into()
                        }
                        line::verbatim::End::VerbatimEscaping(verbatim_escaping) => {
                            top_leaf.state =
                                TopLeafCodeBlockState::InCode(TopLeafCodeBlockStateInCode::Normal);
                            inner.stack.push_top_leaf(top_leaf.into());
                            line::global_phase::process_verbatim_escaping(inner, verbatim_escaping)
                        }
                        line::verbatim::End::Fence => {
                            exit_when_indicator_already_yielded(inner, top_leaf)
                        }
                        line::verbatim::End::None => {
                            top_leaf.state =
                                TopLeafCodeBlockState::InCode(TopLeafCodeBlockStateInCode::Normal);
                            inner.stack.push_top_leaf(top_leaf.into());
                            TYM_UNIT.into()
                        }
                    };

                    tym_a.add(tym_b).add(tym_c)
                }
            };

            Ok(tym)
        }

        fn exit_when_indicator_already_yielded<TStack: Stack<StackEntry>>(
            inner: &mut ParserInner<TStack>,
            top_leaf: TopLeafCodeBlock,
        ) -> Tym<1> {
            inner.r#yield(top_leaf.make_exit_event(inner.current_line()))
        }

        pub fn exit<TStack: Stack<StackEntry>>(
            inner: &mut ParserInner<TStack>,
            top_leaf: TopLeafCodeBlock,
        ) -> Tym<2> {
            let tym_a = if matches!(top_leaf.state, TopLeafCodeBlockState::InInfoString) {
                inner.r#yield(BlockEvent::IndicateCodeBlockCode)
            } else {
                TYM_UNIT.into()
            };

            let tym_b = exit_when_indicator_already_yielded(inner, top_leaf);

            tym_a.add(tym_b)
        }
    }

    pub mod paragraph {
        use super::*;

        /// `content_before` 是已经确认是段落一部分的内容的长度。传入的 `inner.cursor`
        /// 的位置并未加上这些值。调用者要确保如果 `content_before` 不为 0，那
        /// `inner.cursor` 就能作为段落的开头（段落开头应该是并非空格或换行的字符）。
        pub fn enter_if_not_blank<TStack: Stack<StackEntry>>(
            input: &[u8],
            state: &mut State,
            inner: &mut ParserInner<TStack>,
            content_before: usize,
        ) -> crate::Result<Tym<3>> {
            #[cfg(debug_assertions)]
            {
                if content_before > 0 {
                    assert!(!matches!(
                        input.get(inner.cursor()),
                        None | Some(b' ' | b'\r' | b'\n')
                    ))
                }
            }

            // [line::normal::parse] 的过程中可能会涉及到逐字文本转义，导致行数增加，因此
            // 需要提前取得行数。
            let line_start = inner.current_line();

            let has_just_entered_table = inner.has_just_entered_table();
            let (content, mut end) = line::normal::parse(
                input,
                inner,
                line::normal::EndCondition {
                    on_atx_closing: None,
                    on_table_related: branch::braced::table::make_table_related_end_condition(
                        inner,
                        has_just_entered_table,
                    ),
                },
                line::normal::ContentBefore::NotSpace(content_before),
            );

            let tym_ab = if !content.is_empty() || end.is_verbatim_escaping() {
                let id = inner.pop_block_id();
                let top_leaf = TopLeafParagraph {
                    meta: Meta::new(id, line_start),
                    new_line: end.try_take_new_line(),
                };
                let ev = top_leaf.make_enter_event();
                inner.stack.push_top_leaf(top_leaf.into());
                let tym_a = inner.r#yield(ev);

                let tym_b = if !content.is_empty() {
                    inner.r#yield(BlockEvent::Unparsed(content))
                } else {
                    TYM_UNIT.into()
                };

                tym_a.add(tym_b)
            } else {
                TYM_UNIT.into()
            };

            let tym_c = process_normal_end(end, state, inner);

            Ok(tym_ab.add(tym_c))
        }

        fn process_normal_end<TStack: Stack<StackEntry>>(
            end: line::normal::End,
            state: &mut State,
            inner: &mut ParserInner<TStack>,
        ) -> Tym<1> {
            match end {
                line::normal::End::Eof |
                // 段落处理换行是在换行后的那一行，而非换行前的那一行（现在），因此这里跳过处
                // 理。这么做是因为现在不知道下一行还有没有内容，没有内容的话就不应该产出换行
                // 符。此外，这么做也能防止空行产出换行符。
                line::normal::End::NewLine(_) => TYM_UNIT.into(),
                line::normal::End::VerbatimEscaping(verbatim_escaping) => {
                    let tym = line::global_phase::process_verbatim_escaping(inner, verbatim_escaping);
                    cast_tym!(tym)
                }
                line::normal::End::TableRelated(table_related_end) => {
                    let tym = table_related_end.process(state, );
                    cast_tym!(tym)
                }
                line::normal::End::None => TYM_UNIT.into()
            }
        }

        pub fn parse_content_and_process<TStack: Stack<StackEntry>>(
            input: &[u8],
            state: &mut State,
            inner: &mut ParserInner<TStack>,
            mut top_leaf: TopLeafParagraph,
        ) -> crate::Result<Tym<3>> {
            let (mut content, mut end) = line::normal::parse(
                input,
                inner,
                line::normal::EndCondition {
                    on_atx_closing: None,
                    on_table_related: branch::braced::table::make_table_related_end_condition(
                        inner, false,
                    ),
                },
                if inner.current_expecting.spaces_before() > 0 {
                    line::normal::ContentBefore::Space
                } else {
                    line::normal::ContentBefore::NotSpace(0)
                },
            );

            let is_still_in_paragraph =
                !content.is_empty() || top_leaf.new_line.is_none() || end.is_verbatim_escaping();
            let tym_ab = if is_still_in_paragraph {
                let tym_a = if let Some(new_line) = top_leaf.new_line {
                    inner.r#yield(BlockEvent::NewLine(new_line))
                } else {
                    if inner.current_expecting.spaces_before() > 0 {
                        // 在一行的中间，那就不能忽略空格。
                        content.start -= inner.current_expecting.spaces_before();
                    }
                    TYM_UNIT.into()
                };

                top_leaf.new_line = end.try_take_new_line();
                inner.stack.push_top_leaf(top_leaf.into());

                let tym_b = if !content.is_empty() {
                    inner.r#yield(BlockEvent::Unparsed(content))
                } else {
                    TYM_UNIT.into()
                };

                tym_a.add(tym_b)
            } else {
                exit(inner, top_leaf).into()
            };

            let tym_c = process_normal_end(end, state, inner);

            Ok(tym_ab.add(tym_c))
        }

        pub fn exit<TStack: Stack<StackEntry>>(
            inner: &mut ParserInner<TStack>,
            top_leaf: TopLeafParagraph,
        ) -> Tym<1> {
            inner.r#yield(top_leaf.make_exit_event(inner.current_line()))
        }
    }
}
