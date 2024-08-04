use derivative::Derivative;

use super::{
    context::Context,
    global_mapper::Mapped,
    sub_parsers::{self, HaveMet, InTable},
    utils::ArrayQueue,
    BlockInStack, ItemLikeType, Nesting, StackEntry,
};
use crate::{
    block::utils::match_pop_block_id,
    common::{m, Range},
    events::{BlockEvent, BlockWithID, NewLine, ThematicBreak},
    utils::stack::Stack,
};

pub struct Parser<'a> {
    paused_sub_parser: Option<Box<dyn sub_parsers::SubParser<'a> + 'a>>,

    state: State,
    is_new_line: bool,
    to_yield: ArrayQueue<4, BlockEvent>,

    get_nth_item_like_in_stack_memo: Option<GetNthItemLikeInStackMemo>,
}

#[derive(Debug)]
enum State {
    Start { new_line: Option<NewLine> },
    ExpectingContainer,
    ExpectingLeaf { spaces_before: usize },
    ExitingUntil(ExitingUntil),

    Invalid,
}
impl From<InitialState> for State {
    fn from(value: InitialState) -> Self {
        match value {
            InitialState::Start { new_line } => Self::Start { new_line },
            InitialState::ExitingUntil(exiting_until) => Self::ExitingUntil(exiting_until),
        }
    }
}

#[derive(Debug)]
pub enum ExitingUntil {
    ExitedItemLike {
        /// 决定是否一同 exit 最后被 exit 掉的那个 item-like 的容器。为 `true` 代表决定要。
        should_also_exit_container: bool,
        /// 若存在，则在完成 exit 后 enter 指定 item-like。
        and_then_enter_item_like: Option<ItemLikeType>,
    },
    /// 栈顶是 table。
    TopIsTable {
        should_exit_table: bool,
        /// 在完成 exit 后 yield 指定事件。
        and_then_yield: Option<BlockEvent>,
    },
    /// 栈顶知晓 “||”。目前知晓 “||” 的包括 table。
    TopKnownsDoublePipesAndThenYieldSomething,
}

#[derive(Debug)]
pub enum InitialState {
    Start { new_line: Option<NewLine> },
    ExitingUntil(ExitingUntil),
}
impl From<HaveMet> for InitialState {
    fn from(value: HaveMet) -> Self {
        match value {
            HaveMet::None => Self::Start { new_line: None },
            HaveMet::TableClosing => Self::ExitingUntil(ExitingUntil::TopIsTable {
                should_exit_table: true,
                and_then_yield: None,
            }),
            HaveMet::TableCaptionIndicator => Self::ExitingUntil(ExitingUntil::TopIsTable {
                should_exit_table: false,
                and_then_yield: Some(BlockEvent::IndicateTableCaption),
            }),
            HaveMet::TableRowIndicator => Self::ExitingUntil(ExitingUntil::TopIsTable {
                should_exit_table: false,
                and_then_yield: Some(BlockEvent::IndicateTableRow),
            }),
            HaveMet::TableHeaderCellIndicator => Self::ExitingUntil(ExitingUntil::TopIsTable {
                should_exit_table: false,
                and_then_yield: Some(BlockEvent::IndicateTableHeaderCell),
            }),
            HaveMet::DoublePipes => {
                Self::ExitingUntil(ExitingUntil::TopKnownsDoublePipesAndThenYieldSomething)
            }
        }
    }
}

#[derive(Default)]
struct GetNthItemLikeInStackMemo {
    last_n: usize,
    /// 上次返回的那项在 [StackEntry] 中的索引。
    last_index: usize,
}

pub enum ParseStepOutput<'a> {
    ToYield(BlockEvent),
    ToSwitchToSubParser(Box<dyn sub_parsers::SubParser<'a> + 'a>),
    Done,
}

#[derive(Derivative)]
#[derivative(Debug)]
enum InternalOutput<'a> {
    ToContinue(State),
    ToSwitchToSubParser(#[derivative(Debug = "ignore")] Box<dyn sub_parsers::SubParser<'a> + 'a>),
    Done,
}

pub struct NewParserOptions<'a> {
    pub initial_state: InitialState,
    pub paused_sub_parser: Option<Box<dyn sub_parsers::SubParser<'a> + 'a>>,
}

impl<'a> Parser<'a> {
    pub fn new(opts: NewParserOptions<'a>) -> Self {
        Self {
            paused_sub_parser: opts.paused_sub_parser,
            state: opts.initial_state.into(),
            // 这里只是随便初始化一下，实际在 [State::Start] 中决定。
            is_new_line: false,
            to_yield: ArrayQueue::new(),
            get_nth_item_like_in_stack_memo: None,
        }
    }

    pub fn parse<TStack: Stack<StackEntry>>(
        &mut self,
        ctx: &mut Context<'a>,
        stack: &mut TStack,
        nesting: &mut Nesting,
    ) -> crate::Result<ParseStepOutput<'a>> {
        loop {
            if let Some(ev) = self.to_yield.pop_front() {
                return Ok(ParseStepOutput::ToYield(ev));
            }

            #[cfg(debug_assertions)]
            log::debug!("BLOCK->ROOT state={:?}", self.state);

            let internal_result = match std::mem::replace(&mut self.state, State::Invalid) {
                State::Start { new_line } => self.process_in_start_state(ctx, nesting, new_line),
                State::ExpectingContainer => {
                    self.process_in_expecting_container_state(ctx, stack, nesting)?
                }
                State::ExpectingLeaf { spaces_before } => {
                    self.process_in_expecting_leaf_state(ctx, nesting, spaces_before)
                }
                State::ExitingUntil(state) => {
                    self.process_in_exiting_until_state(ctx, stack, nesting, state)
                }
                State::Invalid => unreachable!(),
            };

            #[cfg(debug_assertions)]
            log::debug!("BLOCK->ROOT internal_result={:?}", internal_result);

            match internal_result {
                InternalOutput::ToContinue(state) => self.state = state,
                InternalOutput::ToSwitchToSubParser(sub_parser) => {
                    self.state = State::Invalid;
                    break Ok(ParseStepOutput::ToSwitchToSubParser(sub_parser));
                }
                InternalOutput::Done => {
                    self.state = State::Invalid;
                    break Ok(ParseStepOutput::Done);
                }
            }
        }
    }

    #[inline(always)]
    fn process_in_start_state(
        &mut self,
        ctx: &mut Context<'a>,
        nesting: &mut Nesting,
        new_line: Option<NewLine>,
    ) -> InternalOutput<'a> {
        if let Some(state) = nesting.exiting.take() {
            self.is_new_line = true;
            return InternalOutput::ToContinue(State::ExitingUntil(state));
        }

        #[allow(unused_variables)]
        if let Some(new_line) = new_line {
            self.is_new_line = true;
            #[cfg(feature = "line-number")]
            {
                ctx.current_line_number = new_line.line_number_after;
            }
        } else if let Some(Mapped::NewLine(new_line)) = ctx.mapper.peek(0) {
            self.is_new_line = true;
            #[cfg(feature = "line-number")]
            {
                ctx.current_line_number = new_line.line_number_after;
            }
            if self.is_new_line {
                ctx.must_take_from_mapper_and_apply_to_cursor(1);
            }
        } else {
            self.is_new_line = false
        }

        if self.is_new_line {
            self.get_nth_item_like_in_stack_memo = None;
            nesting.processed_item_likes = 0;
        }

        InternalOutput::ToContinue(State::ExpectingContainer)
    }

    #[inline(always)]
    fn process_in_expecting_container_state<TStack: Stack<StackEntry>>(
        &mut self,
        ctx: &mut Context<'a>,
        stack: &mut TStack,
        nesting: &mut Nesting,
    ) -> crate::Result<InternalOutput<'a>> {
        let is_expecting_deeper = nesting.item_likes_in_stack == nesting.processed_item_likes;
        if is_expecting_deeper {
            if let Some(mut paused_sub_parser) = self.paused_sub_parser.take() {
                paused_sub_parser.resume_from_pause_for_new_line_and_continue(NewLine {
                    #[cfg(feature = "line-number")]
                    line_number_after: ctx.current_line_number,
                });
                return Ok(InternalOutput::ToSwitchToSubParser(paused_sub_parser));
            }
        }

        let spaces = ctx.scan_blank_text().map(|r| r.length()).unwrap_or(0);
        let peeked_3 = ctx.peek_next_three_chars();

        if self.is_new_line {
            if let Some(result) =
                self.process_possible_item_like(ctx, stack, nesting, is_expecting_deeper, &peeked_3)
            {
                return result;
            }
        }
        self.is_new_line = false;

        let output = match try_scan_surrounded_opening(ctx, &peeked_3) {
            TryScanSurroundedResult::TableOpening => {
                nesting.tables_in_stack += 1;
                match_pop_block_id! {
                    ctx,
                    Some(id) => {
                        let ev = BlockEvent::EnterTable(BlockWithID { id });
                        self.to_yield.push_back(ev);
                        stack.try_push(StackEntry {
                            block: BlockInStack::Table,
                            block_id: id,
                            #[cfg(feature = "line-number")]
                            start_line_number: ctx.current_line_number,
                        })?;
                    },
                    None => {
                        let ev = BlockEvent::EnterTable(BlockWithID {});
                        self.to_yield.push_back(ev);
                        stack.try_push(StackEntry {
                            block: BlockInStack::Table,
                            #[cfg(feature = "line-number")]
                            start_line_number: ctx.current_line_number,
                        })?;
                    },
                }

                InternalOutput::ToContinue(State::ExpectingContainer)
            }
            TryScanSurroundedResult::None => InternalOutput::ToContinue(State::ExpectingLeaf {
                spaces_before: spaces,
            }),
        };

        Ok(output)
    }

    #[inline(always)]
    fn process_possible_item_like<TStack: Stack<StackEntry>>(
        &mut self,
        ctx: &mut Context<'a>,
        stack: &mut TStack,
        nesting: &mut Nesting,
        is_expecting_deeper: bool,
        peeked_3: &[Option<u8>; 3],
    ) -> Option<crate::Result<InternalOutput<'a>>> {
        let output = match try_scan_item_like(ctx, peeked_3) {
            TryScanItemLikeResult::None => {
                if is_expecting_deeper {
                    return None;
                } else {
                    InternalOutput::ToContinue(State::ExitingUntil(ExitingUntil::ExitedItemLike {
                        should_also_exit_container: true,
                        and_then_enter_item_like: None,
                    }))
                }
            }
            TryScanItemLikeResult::BlockQuoteLine => {
                nesting.processed_item_likes += 1;
                if is_expecting_deeper {
                    let result = match_pop_block_id! {
                        ctx,
                        Some(id) => {
                            let ev = BlockEvent::EnterBlockQuote(BlockWithID { id });
                            self.to_yield.push_back(ev);
                            stack.try_push(StackEntry {
                                block: BlockInStack::BlockQuote,
                                block_id: id,
                                #[cfg(feature = "line-number")]
                                start_line_number: ctx.current_line_number,
                            })
                        },
                        None => {
                            let ev = BlockEvent::EnterBlockQuote(BlockWithID {});
                            self.to_yield.push_back(ev);
                            stack.try_push(StackEntry {
                                block: BlockInStack::BlockQuote,
                                #[cfg(feature = "line-number")]
                                start_line_number: ctx.current_line_number,
                            })
                        },
                    };
                    if let Err(err) = result {
                        return Some(Err(err));
                    }
                    nesting.item_likes_in_stack += 1;
                }
                InternalOutput::ToContinue(State::ExpectingContainer)
            }
            TryScanItemLikeResult::ItemLike(item_like_type) => {
                if is_expecting_deeper {
                    self.enter_item_like(ctx, stack, nesting, item_like_type, true)
                        .unwrap();
                    InternalOutput::ToContinue(State::ExpectingContainer)
                } else {
                    let last_item_like_type = self.get_nth_item_like_or_paragraph_in_stack(
                        stack.as_slice(),
                        nesting.processed_item_likes,
                    );
                    InternalOutput::ToContinue(State::ExitingUntil(ExitingUntil::ExitedItemLike {
                        should_also_exit_container: !are_item_likes_in_same_group(
                            item_like_type,
                            last_item_like_type,
                        ),
                        and_then_enter_item_like: Some(item_like_type),
                    }))
                }
            }
        };

        Some(Ok(output))
    }

    #[inline(always)]
    fn process_in_expecting_leaf_state(
        &mut self,
        ctx: &mut Context<'a>,
        nesting: &mut Nesting,
        spaces_before: usize,
    ) -> InternalOutput<'a> {
        // XXX: 由于在状态转移到 [State::ExpectingLeaf] 之前一定调用过
        // `ctx.scan_blank_text`，因此 `ctx.mapper.peek_1()` 一定不对应空白字符（
        // “空白字符” 不包含换行。）。

        let Some(peeked) = ctx.mapper.peek(0) else {
            return InternalOutput::Done;
        };
        if peeked.is_new_line() {
            let new_state = State::Start { new_line: None };
            return InternalOutput::ToContinue(new_state);
        }

        match scan_leaf(ctx) {
            LeafType::ThematicBreak => {
                let new_state = State::Start { new_line: None };
                self.to_yield
                    .push_back(BlockEvent::ThematicBreak(ThematicBreak {
                        #[cfg(feature = "block-id")]
                        id: ctx.pop_block_id(),
                        #[cfg(feature = "line-number")]
                        line_number: ctx.current_line_number,
                    }));
                InternalOutput::ToContinue(new_state)
            }
            LeafType::Heading { leading_signs } => InternalOutput::ToSwitchToSubParser(Box::new(
                sub_parsers::heading::Parser::new(sub_parsers::heading::NewParserOptions {
                    #[cfg(feature = "line-number")]
                    start_line_number: ctx.current_line_number,
                    leading_signs,
                    in_table: if nesting.tables_in_stack > 0 {
                        Some(InTable {
                            has_yielded_since_entered: nesting.has_yielded_since_entered_last_table,
                        })
                    } else {
                        None
                    },
                }),
            )),
            LeafType::CodeBlock { backticks } => InternalOutput::ToSwitchToSubParser(Box::new(
                sub_parsers::code_block::Parser::new(sub_parsers::code_block::NewParserOptions {
                    #[cfg(feature = "line-number")]
                    start_line_number: ctx.current_line_number,
                    leading_backticks: backticks,
                    indentation: spaces_before,
                }),
            )),
            LeafType::Paragraph { content_before } => {
                InternalOutput::ToSwitchToSubParser(Box::new(sub_parsers::paragraph::Parser::new(
                    sub_parsers::paragraph::NewParserOptions {
                        #[cfg(feature = "line-number")]
                        start_line_number: ctx.current_line_number,
                        content_before,
                        in_table: if nesting.tables_in_stack > 0 {
                            Some(InTable {
                                has_yielded_since_entered: nesting
                                    .has_yielded_since_entered_last_table,
                            })
                        } else {
                            None
                        },
                    },
                )))
            }
        }
    }

    #[inline(always)]
    fn process_in_exiting_until_state<TStack: Stack<StackEntry>>(
        &mut self,
        ctx: &mut Context<'a>,
        stack: &mut TStack,
        nesting: &mut Nesting,
        state: ExitingUntil,
    ) -> InternalOutput<'a> {
        if let Some(mut sub_parser) = self.paused_sub_parser.take() {
            nesting.exiting = Some(state);
            sub_parser.resume_from_pause_for_new_line_and_exit();
            return InternalOutput::ToSwitchToSubParser(sub_parser);
        }

        let top = stack.pop().unwrap();
        match top.block {
            BlockInStack::ItemLike { .. } | BlockInStack::BlockQuote { .. } => {
                nesting.item_likes_in_stack -= 1;
                if nesting.processed_item_likes == nesting.item_likes_in_stack {
                    let is_item_like = top.block.is_item_like();
                    if let ExitingUntil::ExitedItemLike {
                        should_also_exit_container,
                        and_then_enter_item_like,
                    } = state
                    {
                        self.to_yield
                            .push_back(BlockEvent::ExitBlock(top.into_exit_block(
                                #[cfg(feature = "line-number")]
                                ctx.current_line_number,
                            )));
                        if is_item_like && should_also_exit_container {
                            let top = stack.pop().unwrap();
                            self.to_yield
                                .push_back(BlockEvent::ExitBlock(top.into_exit_block(
                                    #[cfg(feature = "line-number")]
                                    ctx.current_line_number,
                                )));
                        }
                        if let Some(item_like_type_to_enter) = and_then_enter_item_like {
                            self.enter_item_like(
                                ctx,
                                stack,
                                nesting,
                                item_like_type_to_enter,
                                should_also_exit_container,
                            )
                            .unwrap();
                        }
                        return InternalOutput::ToContinue(State::ExpectingContainer);
                    }
                }
            }
            BlockInStack::Table => match state {
                ExitingUntil::ExitedItemLike { .. } => {
                    nesting.tables_in_stack -= 1;
                }
                ExitingUntil::TopIsTable {
                    should_exit_table,
                    and_then_yield,
                } => {
                    if let Some(to_yield) = and_then_yield {
                        self.to_yield.push_back(to_yield);
                    }
                    if should_exit_table {
                        nesting.tables_in_stack -= 1;
                        self.to_yield
                            .push_back(BlockEvent::ExitBlock(top.into_exit_block(
                                #[cfg(feature = "line-number")]
                                ctx.current_line_number,
                            )));
                    } else {
                        stack.try_push(top).unwrap();
                    }
                    return InternalOutput::ToContinue(State::ExpectingContainer);
                }
                ExitingUntil::TopKnownsDoublePipesAndThenYieldSomething => {
                    stack.try_push(top).unwrap();
                    self.to_yield.push_back(BlockEvent::IndicateTableDataCell);
                    return InternalOutput::ToContinue(State::ExpectingContainer);
                }
            },
            BlockInStack::Container => {}
        }
        self.to_yield
            .push_back(BlockEvent::ExitBlock(top.into_exit_block(
                #[cfg(feature = "line-number")]
                ctx.current_line_number,
            )));
        InternalOutput::ToContinue(State::ExitingUntil(state))
    }

    /// `n` 基于 0。
    ///
    /// 返回 [None] 代表对应位置的是段落而非 item-like。
    fn get_nth_item_like_or_paragraph_in_stack(
        &mut self,
        stack: &[StackEntry],
        n: usize,
    ) -> Option<ItemLikeType> {
        let memo = self
            .get_nth_item_like_in_stack_memo
            .take()
            .unwrap_or_default();

        let mut n_countdown = n - memo.last_n;

        for (delta_index, entry) in stack.iter().skip(memo.last_index).enumerate() {
            let mut item_like_type: Option<ItemLikeType> = None;
            if let BlockInStack::ItemLike { typ, .. } = entry.block {
                item_like_type = Some(typ);
            } else if !matches!(entry.block, BlockInStack::BlockQuote { .. }) {
                continue;
            }

            if n_countdown == 0 {
                self.get_nth_item_like_in_stack_memo = Some(GetNthItemLikeInStackMemo {
                    last_n: n,
                    last_index: memo.last_index + delta_index,
                });
                return item_like_type;
            }
            n_countdown -= 1;
        }
        unreachable!()
    }

    fn enter_item_like<TStack: Stack<StackEntry>>(
        &mut self,
        #[allow(unused_variables)] ctx: &mut Context<'a>,
        stack: &mut TStack,
        nesting: &mut Nesting,
        item_like_type: ItemLikeType,
        should_enter_container: bool,
    ) -> crate::Result<()> {
        if !matches!(
            item_like_type,
            ItemLikeType::OrderedListItem
                | ItemLikeType::UnorderedListItem
                | ItemLikeType::DescriptionTerm
                | ItemLikeType::DescriptionDetails
        ) {
            unreachable!()
        }

        nesting.processed_item_likes += 1;
        if should_enter_container {
            match_pop_block_id! {
                ctx,
                Some(id) => {
                    stack.try_push(StackEntry {
                        block: BlockInStack::Container,
                        block_id: id,
                        #[cfg(feature = "line-number")]
                        start_line_number: ctx.current_line_number,
                    })?;
                    self.to_yield.push_back(item_like_type.into_enter_container_block_event(id));
                },
                None => {
                    stack.try_push(StackEntry {
                        block: BlockInStack::Container,
                        #[cfg(feature = "line-number")]
                        start_line_number: ctx.current_line_number,
                    })?;
                    self.to_yield.push_back(item_like_type.into_enter_container_block_event());
                },
            }
        }

        nesting.item_likes_in_stack += 1;
        match_pop_block_id! {
            ctx,
            Some(id) => {
                stack.try_push(StackEntry {
                    block: item_like_type.into(),
                    block_id: id,
                    #[cfg(feature = "line-number")]
                    start_line_number: ctx.current_line_number,
                })?;
                self.to_yield.push_back(item_like_type.into_enter_block_event(id));
            },
            None => {
                stack.try_push(StackEntry {
                    block: item_like_type.into(),
                    #[cfg(feature = "line-number")]
                    start_line_number: ctx.current_line_number,
                })?;
                self.to_yield.push_back(item_like_type.into_enter_block_event());
            },
        };

        Ok(())
    }
}

enum TryScanItemLikeResult {
    BlockQuoteLine,
    ItemLike(ItemLikeType),
    None,
}
#[inline(always)]
fn try_scan_item_like(ctx: &mut Context, peeked_3: &[Option<u8>; 3]) -> TryScanItemLikeResult {
    match peeked_3 {
        [Some(m!('>')), ref second_char, ..] if check_is_indeed_item_like(ctx, second_char) => {
            return TryScanItemLikeResult::BlockQuoteLine;
        }
        [Some(m!('#')), ref second_char, ..] if check_is_indeed_item_like(ctx, second_char) => {
            return TryScanItemLikeResult::ItemLike(ItemLikeType::OrderedListItem);
        }
        [Some(m!('*')), ref second_char, ..] if check_is_indeed_item_like(ctx, second_char) => {
            return TryScanItemLikeResult::ItemLike(ItemLikeType::UnorderedListItem);
        }
        [Some(m!(';')), ref second_char, ..] if check_is_indeed_item_like(ctx, second_char) => {
            return TryScanItemLikeResult::ItemLike(ItemLikeType::DescriptionTerm);
        }
        [Some(m!(':')), ref second_char, ..] if check_is_indeed_item_like(ctx, second_char) => {
            return TryScanItemLikeResult::ItemLike(ItemLikeType::DescriptionDetails);
        }
        _ => {}
    };
    TryScanItemLikeResult::None
}

enum TryScanSurroundedResult {
    TableOpening,
    None,
}
#[inline(always)]
fn try_scan_surrounded_opening(
    ctx: &mut Context,
    peeked_3: &[Option<u8>; 3],
) -> TryScanSurroundedResult {
    match peeked_3 {
        [Some(m!('{')), Some(m!('|')), ..] => {
            ctx.must_take_from_mapper_and_apply_to_cursor(2);
            TryScanSurroundedResult::TableOpening
        }
        _ => TryScanSurroundedResult::None,
    }
}

fn check_is_indeed_item_like(ctx: &mut Context, second_char: &Option<u8>) -> bool {
    let is_indeed_item_like = second_char == &Some(b' ') || {
        let p = ctx.mapper.peek(1);
        p.is_none() || p.is_some_and(|p| p.is_new_line())
    };
    if is_indeed_item_like {
        let to_take = if second_char.is_some() { 2 } else { 1 };
        ctx.must_take_from_mapper_and_apply_to_cursor(to_take);
        true
    } else {
        false
    }
}

enum LeafType {
    ThematicBreak,
    Heading { leading_signs: usize },
    CodeBlock { backticks: usize },
    Paragraph { content_before: Option<Range> },
}

#[inline(always)]
fn scan_leaf(ctx: &mut Context) -> LeafType {
    match ctx.peek_next_three_chars() {
        [Some(m!('-')), Some(m!('-')), Some(m!('-'))] => {
            ctx.must_take_from_mapper_and_apply_to_cursor(3);
            ctx.drop_from_mapper_while_char(m!('-'));
            LeafType::ThematicBreak
        }
        [Some(m!('=')), ..] => {
            ctx.must_take_from_mapper_and_apply_to_cursor(1);
            let mut potential_opening_part = Range::new(ctx.cursor.value().unwrap(), 1);
            let dropped = ctx.drop_from_mapper_while_char_with_maximum(m!('='), 5);
            potential_opening_part.increase_length(dropped);

            if ctx.peek_next_char() == Some(b' ') {
                ctx.must_take_from_mapper_and_apply_to_cursor(1);
                let leading_signs = 1 + dropped;
                LeafType::Heading { leading_signs }
            } else {
                LeafType::Paragraph {
                    content_before: Some(potential_opening_part),
                }
            }
        }
        [Some(m!('`')), Some(m!('`')), Some(m!('`'))] => {
            ctx.must_take_from_mapper_and_apply_to_cursor(3);
            let extra_count = ctx.drop_from_mapper_while_char(m!('`'));
            LeafType::CodeBlock {
                backticks: 3 + extra_count,
            }
        }
        _ => LeafType::Paragraph {
            content_before: None,
        },
    }
}

#[inline(always)]
fn are_item_likes_in_same_group(left: ItemLikeType, right: Option<ItemLikeType>) -> bool {
    let Some(right) = right else {
        return false;
    };
    match left {
        ItemLikeType::OrderedListItem | ItemLikeType::UnorderedListItem => left == right,
        ItemLikeType::DescriptionTerm | ItemLikeType::DescriptionDetails => matches!(
            right,
            ItemLikeType::DescriptionTerm | ItemLikeType::DescriptionDetails
        ),
    }
}
