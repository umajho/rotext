mod line;
mod parser_inner;
mod stack_wrapper;
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

    #[cfg(debug_assertions)]
    is_errored: bool,
}

impl<'a, TStack: Stack<StackEntry>> Parser<'a, TStack> {
    pub fn new(input: &'a [u8]) -> Self {
        Self {
            input,
            state: Expecting::ItemLikeOpening.into(),
            inner: ParserInner::new(),
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
                State::Exiting(exiting_branches) => {
                    match Self::exit(&mut self.inner, exiting_branches) {
                        Ok((tym, state)) => {
                            if let Some(state) = state {
                                self.state = state;
                            }
                            Ok(tym)
                        }
                        Err(err) => Err(err),
                    }
                    .map(|tym| cast_tym!(tym))
                }
                State::Ended => {
                    break None;
                }
                State::Expecting(expecting) => {
                    let Some(&first_char) = self.input.get(self.inner.cursor()) else {
                        self.state = if self.inner.stack.is_empty() {
                            State::Ended
                        } else {
                            Exiting::new(ExitingUntil::StackIsEmpty, ExitingAndThen::End).into()
                        };

                        continue;
                    };
                    let expecting = *expecting;
                    self.parse(expecting, first_char)
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
    fn parse(&mut self, expecting: Expecting, first_char: u8) -> crate::Result<Tym<3>> {
        match expecting {
            Expecting::ItemLikeOpening => branch::item_like::parse_opening_and_process(
                self.input,
                &mut self.state,
                &mut self.inner,
                first_char,
            )
            .map(|tym| cast_tym!(tym)),
            Expecting::SurroundedOpening => branch::surrounded::parse_opening_and_process(
                self.input,
                &mut self.state,
                &mut self.inner,
                first_char,
            )
            .map(|tym| cast_tym!(tym)),
        }
    }

    fn exit(
        inner: &mut ParserInner<TStack>,
        exiting: &mut Exiting,
    ) -> crate::Result<(Tym<2>, Option<State>)> {
        if let Some(top_leaf) = inner.stack.pop_top_leaf() {
            let result = match top_leaf {
                TopLeaf::Paragraph(top_leaf) => leaf::paragraph::exit(inner, top_leaf),
                TopLeaf::Heading(_top_leaf) => unreachable!(),
                TopLeaf::CodeBlock(top_leaf) => leaf::code_block::exit(inner, top_leaf),
            };
            return result.map(|tym| (cast_tym!(tym), None));
        }

        let is_done = match exiting.until {
            ExitingUntil::OnlyNItemLikesRemain {
                n,
                should_also_exit_containee_in_last_container,
            } => {
                debug_assert!(inner.stack.item_likes_in_stack() >= n);
                let mut is_done = inner.stack.item_likes_in_stack() == n;
                if should_also_exit_containee_in_last_container {
                    is_done = is_done && inner.stack.top_is_item_like_container();
                }
                is_done
            }
            ExitingUntil::StackIsEmpty => inner.stack.is_empty(),
        };

        if is_done {
            debug_assert!(exiting.and_then.is_some());
            return match unsafe { exiting.and_then.take().unwrap_unchecked() } {
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
                    Ok((tym_a.add(tym_b), Some(Expecting::ItemLikeOpening.into())))
                }
                ExitingAndThen::ExpectSurroundedOpening => {
                    Ok((TYM_UNIT.into(), Some(Expecting::SurroundedOpening.into())))
                }
                ExitingAndThen::End => Ok((TYM_UNIT.into(), Some(State::Ended))),
            };
        }

        let result = match inner.stack.pop().unwrap() {
            StackEntry::ItemLike(stack_entry) => branch::item_like::exit(inner, stack_entry),
            StackEntry::ItemLikeContainer(stack_entry) => {
                branch::item_like::exit_container(inner, stack_entry)
            }
            StackEntry::Table(stack_entry) => branch::surrounded::table::exit(inner, stack_entry),
        };
        result.map(|tym| (cast_tym!(tym), None))
    }
}

impl<'a, TStack: Stack<StackEntry>> Iterator for Parser<'a, TStack> {
    type Item = crate::Result<BlockEvent>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next()
    }
}

enum State {
    Expecting(Expecting),
    /// 持续从栈中推出内容并产出对应的退出事件，直到满足特定条件，在那之后执行要做的事情。
    Exiting(Exiting),
    Ended,
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

#[derive(Clone, Copy)]
enum Expecting {
    ItemLikeOpening,
    SurroundedOpening,
}

struct Exiting {
    until: ExitingUntil,
    /// 完成退出后要做什么。
    ///
    /// XXX: 必定为 `Some`，因为在它被 `take` 走后 parser 的状态必定被新的状态覆盖。这里
    /// 使用 Option 仅仅是为了 workaround rust 的生命周期。
    and_then: Option<ExitingAndThen>,
}
enum ExitingUntil {
    OnlyNItemLikesRemain {
        n: usize,
        should_also_exit_containee_in_last_container: bool,
    },
    StackIsEmpty,
}
enum ExitingAndThen {
    EnterItemLikeAndExpectItemLike {
        container: Option<StackEntryItemLikeContainer>,
        item_like: StackEntryItemLike,
    },
    ExpectSurroundedOpening,
    End,
}
impl Exiting {
    fn new(until: ExitingUntil, and_then: ExitingAndThen) -> Self {
        Self {
            until,
            and_then: Some(and_then),
        }
    }
}

mod branch {
    use super::*;

    pub mod item_like {

        use super::*;

        macro_rules! return_parse_paragraph_if_no_following_space {
            ($input:expr, $inner:expr) => {
                if !matches!($input.get($inner.cursor() + 1), Some(b' ')) {
                    return leaf::paragraph::enter_if_not_blank($input, $inner, 1)
                        .map(|tym| cast_tym!(tym));
                }
            };
        }

        pub fn parse_opening_and_process<TStack: Stack<StackEntry>>(
            input: &[u8],
            state: &mut State,
            inner: &mut ParserInner<TStack>,
            first_char: u8,
        ) -> crate::Result<Tym<3>> {
            if !inner.stack.has_unprocessed_item_likes_at_current_line() {
                if let Some(top_leaf) = inner.stack.pop_top_leaf() {
                    return leaf::parse_content_and_process(input, inner, top_leaf)
                        .map(|tym| cast_tym!(tym));
                }
            }

            use GeneralItemLike as I;
            use ItemLikeContainer as G;

            match first_char {
                m!('>') => {
                    return_parse_paragraph_if_no_following_space!(input, inner);
                    inner.move_cursor_forward("> ".len());
                    process_greater_than_opening(inner).map(|tym| cast_tym!(tym))
                }
                m!('#') => {
                    return_parse_paragraph_if_no_following_space!(input, inner);
                    inner.move_cursor_forward("# ".len());
                    process_general_opening(state, inner, G::OL, I::LI).map(|tym| cast_tym!(tym))
                }
                m!('*') => {
                    return_parse_paragraph_if_no_following_space!(input, inner);
                    inner.move_cursor_forward("* ".len());
                    process_general_opening(state, inner, G::UL, I::LI).map(|tym| cast_tym!(tym))
                }
                m!(';') => {
                    return_parse_paragraph_if_no_following_space!(input, inner);
                    inner.move_cursor_forward("; ".len());
                    process_general_opening(state, inner, G::DL, I::DT).map(|tym| cast_tym!(tym))
                }
                m!(':') => {
                    return_parse_paragraph_if_no_following_space!(input, inner);
                    inner.move_cursor_forward(": ".len());
                    process_general_opening(state, inner, G::DL, I::DD).map(|tym| cast_tym!(tym))
                }
                _ if inner.stack.has_unprocessed_item_likes_at_current_line() => {
                    *state = Exiting::new(
                        ExitingUntil::OnlyNItemLikesRemain {
                            n: inner.stack.processed_item_likes_at_current_line(),
                            should_also_exit_containee_in_last_container: false,
                        },
                        ExitingAndThen::ExpectSurroundedOpening,
                    )
                    .into();
                    Ok(TYM_UNIT.into())
                }
                _ => surrounded::parse_opening_and_process(input, state, inner, first_char)
                    .map(|tym| cast_tym!(tym)),
            }
        }

        fn process_greater_than_opening<TStack: Stack<StackEntry>>(
            inner: &mut ParserInner<TStack>,
        ) -> crate::Result<Tym<1>> {
            let tym = if inner.stack.has_unprocessed_item_likes_at_current_line() {
                inner
                    .stack
                    .mark_first_unprocessed_item_like_as_processed_at_current_line();

                TYM_UNIT.into()
            } else {
                let id = inner.pop_block_id();
                inner
                    .stack
                    .push_item_like_container(StackEntryItemLikeContainer {
                        meta: Meta::new(id, inner.current_line()),
                        r#type: ItemLikeContainer::BlockQuote,
                    })?;
                inner.r#yield(BlockEvent::EnterBlockQuote(id.into()))
            };

            Ok(tym)
        }

        fn process_general_opening<TStack: Stack<StackEntry>>(
            state: &mut State,
            inner: &mut ParserInner<TStack>,
            container: ItemLikeContainer,
            item_like: GeneralItemLike,
        ) -> crate::Result<Tym<2>> {
            let tym = if inner.stack.has_unprocessed_item_likes_at_current_line() {
                let stack_entry = inner.stack.first_unprocessed_item_like_at_current_line();
                if stack_entry.r#type == container {
                    inner
                        .stack
                        .mark_first_unprocessed_item_like_as_processed_at_current_line();
                    *state = Exiting::new(
                        ExitingUntil::OnlyNItemLikesRemain {
                            n: inner.stack.processed_item_likes_at_current_line(),
                            should_also_exit_containee_in_last_container: true,
                        },
                        ExitingAndThen::EnterItemLikeAndExpectItemLike {
                            container: None,
                            item_like: make_stack_entry_from_general_item_like(item_like, inner),
                        },
                    )
                    .into()
                } else {
                    *state = Exiting::new(
                        ExitingUntil::OnlyNItemLikesRemain {
                            n: inner.stack.processed_item_likes_at_current_line(),
                            should_also_exit_containee_in_last_container: false,
                        },
                        ExitingAndThen::EnterItemLikeAndExpectItemLike {
                            container: Some(make_stack_entry_from_item_like_container(
                                container, inner,
                            )),
                            item_like: make_stack_entry_from_general_item_like(item_like, inner),
                        },
                    )
                    .into()
                }

                TYM_UNIT.into()
            } else {
                let tym_a = {
                    let stack_entry = make_stack_entry_from_item_like_container(container, inner);
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
            };

            Ok(tym)
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

    pub mod surrounded {
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
                    _ => leaf::paragraph::enter_if_not_blank(input, inner, 1)
                        .map(|tym| cast_tym!(tym)),
                },
                _ => leaf::parse_opening_and_process(input, inner, first_char)
                    .map(|tym| cast_tym!(tym)),
            }
        }

        pub mod table {
            use super::*;

            pub(super) fn enter<TStack: Stack<StackEntry>>(
                state: &mut State,
                inner: &mut ParserInner<TStack>,
            ) -> crate::Result<Tym<1>> {
                *state = Expecting::SurroundedOpening.into();

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
                pub fn process(self) -> Tym<0> {
                    todo!()
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
        }
    }
}

mod leaf {
    use super::*;

    pub fn parse_opening_and_process<TStack: Stack<StackEntry>>(
        input: &[u8],
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
                    paragraph::enter_if_not_blank(input, inner, count).map(|tym| cast_tym!(tym))
                }
            }
            m!('=') => {
                let count = 1 + count_continuous_character(input, m!('='), inner.cursor() + 1);
                if (1..=6).contains(&count) && input.get(inner.cursor() + count) == Some(&b' ') {
                    inner.move_cursor_forward(count + " ".len());
                    heading::enter(inner, count).map(|tym| cast_tym!(tym))
                } else {
                    paragraph::enter_if_not_blank(input, inner, count).map(|tym| cast_tym!(tym))
                }
            }
            m!('`') => {
                let count = 1 + count_continuous_character(input, m!('`'), inner.cursor() + 1);
                if count >= 3 {
                    code_block::enter(inner, count).map(|tym| cast_tym!(tym))
                } else {
                    paragraph::enter_if_not_blank(input, inner, count).map(|tym| cast_tym!(tym))
                }
            }
            _ => paragraph::enter_if_not_blank(input, inner, 0).map(|tym| cast_tym!(tym)),
        }
    }

    pub fn parse_content_and_process<TStack: Stack<StackEntry>>(
        input: &[u8],
        inner: &mut ParserInner<TStack>,
        top_leaf: TopLeaf,
    ) -> crate::Result<Tym<3>> {
        match top_leaf {
            TopLeaf::Paragraph(top_leaf) => {
                leaf::paragraph::parse_content_and_process(input, inner, top_leaf)
                    .map(|tym| cast_tym!(tym))
            }
            TopLeaf::Heading(top_leaf) => {
                leaf::heading::parse_content_and_process(input, inner, top_leaf)
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
        use super::*;

        pub(super) fn enter<TStack: Stack<StackEntry>>(
            inner: &mut ParserInner<TStack>,
            level: usize,
        ) -> crate::Result<Tym<1>> {
            let id = inner.pop_block_id();
            let top_leaf = TopLeafHeading {
                meta: Meta::new(id, inner.current_line()),
                level,
            };
            let ev = top_leaf.make_event();
            inner.stack.push_top_leaf(top_leaf.into());
            let tym = inner.r#yield(ev);

            Ok(tym)
        }

        pub fn parse_content_and_process<TStack: Stack<StackEntry>>(
            input: &[u8],
            inner: &mut ParserInner<TStack>,
            top_leaf: TopLeafHeading,
        ) -> crate::Result<Tym<2>> {
            todo!()
        }
    }

    pub mod code_block {
        use stack_wrapper::TopLeafCodeBlockState;

        use super::*;

        pub(super) fn enter<TStack: Stack<StackEntry>>(
            inner: &mut ParserInner<TStack>,
            backticks: usize,
        ) -> crate::Result<Tym<1>> {
            let id = inner.pop_block_id();
            let top_leaf = TopLeafCodeBlock {
                meta: Meta::new(id, inner.current_line()),
                backticks,
                state: TopLeafCodeBlockState::ExpectingInfoString,
            };
            let ev = top_leaf.make_enter_event();
            inner.stack.push_top_leaf(top_leaf.into());
            let tym = inner.r#yield(ev);

            Ok(tym)
        }

        pub fn parse_content_and_process<TStack: Stack<StackEntry>>(
            input: &[u8],
            inner: &mut ParserInner<TStack>,
            top_leaf: TopLeafCodeBlock,
        ) -> crate::Result<Tym<2>> {
            todo!("info string");

            todo!()
        }

        pub fn exit<TStack: Stack<StackEntry>>(
            inner: &mut ParserInner<TStack>,
            top_leaf: TopLeafCodeBlock,
        ) -> crate::Result<Tym<1>> {
            let tym = inner.r#yield(top_leaf.make_exit_event(inner.current_line()));

            Ok(tym)
        }
    }

    pub mod paragraph {
        use super::*;

        /// `content_before` 是已经确认是段落一部分的内容的长度。传入的 `inner.cursor`
        /// 的位置并未加上这些值。调用者要确保如果 `content_before` 不为 0，那
        /// `inner.cursor` 就能作为段落的开头（段落开头应该是并非空格或换行的字符）。
        pub fn enter_if_not_blank<TStack: Stack<StackEntry>>(
            input: &[u8],
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

            let (content, mut end) = line::normal::parse(
                input,
                inner,
                line::normal::EndCondition {
                    on_atx_closing: None,
                    on_table_related: if inner.stack.tables_in_stack() > 0 {
                        Some(line::normal::TableRelated {
                            is_caption_applicable: todo!(),
                        })
                    } else {
                        None
                    },
                },
                content_before,
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

            let tym_c = process_normal_end(end, inner);

            Ok(tym_ab.add(tym_c))
        }

        fn process_normal_end<TCtx: YieldContext>(
            end: line::normal::End,
            ctx: &mut TCtx,
        ) -> Tym<1> {
            match end {
                line::normal::End::Eof |
                // 段落处理换行是在换行后的那一行，而非换行前的那一行（现在），因此这里跳过处
                // 理。这么做是因为现在不知道下一行还有没有内容，没有内容的话就不应该产出换行
                // 符。此外，这么做也能防止空行产出换行符。
                line::normal::End::NewLine(_) => TYM_UNIT.into(),
                line::normal::End::VerbatimEscaping(verbatim_escaping) => {
                    let tym = line::global_phase::process_verbatim_escaping(ctx, verbatim_escaping);
                    cast_tym!(tym)
                }
                line::normal::End::TableRelated(table_related_end) => {
                    let tym = table_related_end.process();
                    cast_tym!(tym)
                }
            }
        }

        pub fn parse_content_and_process<TStack: Stack<StackEntry>>(
            input: &[u8],
            inner: &mut ParserInner<TStack>,
            mut top_leaf: TopLeafParagraph,
        ) -> crate::Result<Tym<3>> {
            let (content, mut end) = line::normal::parse(
                input,
                inner,
                line::normal::EndCondition {
                    on_atx_closing: None,
                    on_table_related: if inner.stack.tables_in_stack() > 0 {
                        Some(line::normal::TableRelated {
                            is_caption_applicable: false,
                        })
                    } else {
                        None
                    },
                },
                0,
            );

            let tym_ab = if !content.is_empty() || end.is_verbatim_escaping() {
                let tym_a = if let Some(new_line) = top_leaf.new_line {
                    inner.r#yield(BlockEvent::NewLine(new_line))
                } else {
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
                inner
                    .r#yield(top_leaf.make_exit_event(inner.current_line()))
                    .into()
            };

            let tym_c = process_normal_end(end, inner);

            Ok(tym_ab.add(tym_c))
        }

        pub fn exit<TStack: Stack<StackEntry>>(
            inner: &mut ParserInner<TStack>,
            top_leaf: TopLeafParagraph,
        ) -> crate::Result<Tym<1>> {
            let tym = inner.r#yield(top_leaf.make_exit_event(inner.current_line()));
            Ok(tym)
        }
    }
}
