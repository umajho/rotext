use derivative::Derivative;

use super::{context::Context, sub_parsers, ItemLikeType, Nesting, StackEntry};
use crate::{common::Range, events::BlockEvent};

pub struct Parser<'a> {
    paused_sub_parser: Option<Box<dyn sub_parsers::SubParser<'a> + 'a>>,

    state: State,
    is_new_line: bool,
    deferred_to_yield: Option<ToYield>,

    get_nth_item_like_in_stack_memo: Option<GetNthItemLikeInStackMemo>,
}

#[derive(Debug)]
enum State {
    Start {
        is_certain_is_new_line: Option<bool>,
    },
    ExpectingContainer,
    ExpectingLeaf,
    ExitingDiscontinuedItemLikes(ExitingDiscontinuedItemLikesState),

    Invalid,
}
#[derive(Debug)]
pub struct ExitingDiscontinuedItemLikesState {
    /// 决定是否一同 exit 最后被 exit 掉的那个 item-like 的容器。为 `true` 代表决定要。
    should_keep_container: bool,
    /// 若存在，则在完成 exit 后 enter 指定 item-like。
    and_then_enter_item_like: Option<ItemLikeType>,
}

#[derive(Debug)]
enum ToYield {
    One(BlockEvent),
    Two(BlockEvent, BlockEvent),
}

#[derive(Default)]
struct GetNthItemLikeInStackMemo {
    last_n: usize,
    /// 上次返回的那项在 [StackEntry] 中的索引。
    last_index: usize,
}

pub enum Result<'a> {
    ToYield(BlockEvent),
    ToSwitchToSubParser(Box<dyn sub_parsers::SubParser<'a> + 'a>),
    Done,
}

#[derive(Derivative)]
#[derivative(Debug)]
enum InternalResult<'a> {
    ToYield(State, ToYield),
    ToContinue(State),
    ToSwitchToSubParser(#[derivative(Debug = "ignore")] Box<dyn sub_parsers::SubParser<'a> + 'a>),
    Done,
}

impl<'a> Parser<'a> {
    pub fn new(
        is_certain_is_new_line: Option<bool>,
        paused_sub_parser: Option<Box<dyn sub_parsers::SubParser<'a> + 'a>>,
    ) -> Self {
        Self {
            paused_sub_parser,
            state: State::Start {
                is_certain_is_new_line,
            },
            // 这里只是随便初始化一下，实际在 [State::Start] 中决定。
            is_new_line: false,
            deferred_to_yield: None,
            get_nth_item_like_in_stack_memo: None,
        }
    }

    pub fn parse(
        &mut self,
        ctx: &mut Context<'a>,
        stack: &mut Vec<StackEntry>,
        nesting: &mut Nesting,
    ) -> Result<'a> {
        if let Some(to_yield) = self.deferred_to_yield.take() {
            match to_yield {
                ToYield::One(ev) => {
                    return Result::ToYield(ev);
                }
                ToYield::Two(ev_1, ev_2) => {
                    self.deferred_to_yield = Some(ToYield::One(ev_2));
                    return Result::ToYield(ev_1);
                }
            }
        }

        loop {
            // log::debug!("ROOT state={:?}", self.state);

            let internal_result = match std::mem::replace(&mut self.state, State::Invalid) {
                State::Start {
                    is_certain_is_new_line: is_new_line,
                } => self.process_in_start_state(ctx, nesting, is_new_line),
                State::ExpectingContainer => {
                    self.process_in_expecting_container_state(ctx, stack, nesting)
                }
                State::ExpectingLeaf => self.process_in_expecting_leaf_state(ctx),
                State::ExitingDiscontinuedItemLikes(state) => {
                    self.process_in_exiting_discontinued_item_likes_state(stack, nesting, state)
                }
                State::Invalid => unreachable!(),
            };

            // log::debug!("ROOT internal_result={:?}", internal_result);

            match internal_result {
                InternalResult::ToYield(state, ToYield::One(ev)) => {
                    self.state = state;
                    break Result::ToYield(ev);
                }
                InternalResult::ToYield(state, ToYield::Two(ev_1, ev_2)) => {
                    self.state = state;
                    self.deferred_to_yield = Some(ToYield::One(ev_2));
                    break Result::ToYield(ev_1);
                }
                InternalResult::ToContinue(state) => self.state = state,
                InternalResult::ToSwitchToSubParser(sub_parser) => {
                    self.state = State::Invalid;
                    break Result::ToSwitchToSubParser(sub_parser);
                }
                InternalResult::Done => {
                    self.state = State::Invalid;
                    break Result::Done;
                }
            }
        }
    }

    #[inline(always)]
    fn process_in_start_state(
        &mut self,
        ctx: &mut Context<'a>,
        nesting: &mut Nesting,
        is_certain_is_new_line: Option<bool>,
    ) -> InternalResult<'a> {
        if let Some(state) = nesting.is_exiting_discontinued_item_likes.take() {
            return InternalResult::ToContinue(State::ExitingDiscontinuedItemLikes(state));
        }

        if let Some(is_new_line) = is_certain_is_new_line {
            self.is_new_line = is_new_line;
            self.get_nth_item_like_in_stack_memo = None;
        } else {
            self.is_new_line = ctx.mapper.peek_1().is_some_and(|p| p.is_line_feed());
            if self.is_new_line {
                ctx.must_take_from_mapper_and_apply_to_cursor(1);
                nesting.processed_item_likes = 0;
            }
        }

        InternalResult::ToContinue(State::ExpectingContainer)
    }

    #[inline(always)]
    fn process_in_expecting_container_state(
        &mut self,
        ctx: &mut Context<'a>,
        stack: &mut Vec<StackEntry>,
        nesting: &mut Nesting,
    ) -> InternalResult<'a> {
        let is_expecting_deeper = nesting.item_likes_in_stack == nesting.processed_item_likes;
        if is_expecting_deeper {
            if let Some(mut paused_sub_parser) = self.paused_sub_parser.take() {
                paused_sub_parser.resume_from_pause_for_new_line_and_continue();
                return InternalResult::ToSwitchToSubParser(paused_sub_parser);
            }
        }

        _ = ctx.scan_blank_text();
        if self.is_new_line {
            let Some(item_like_type) = scan_item_like(ctx) else {
                if is_expecting_deeper {
                    return InternalResult::ToContinue(State::ExpectingLeaf);
                } else {
                    return InternalResult::ToContinue(State::ExitingDiscontinuedItemLikes(
                        ExitingDiscontinuedItemLikesState {
                            should_keep_container: false,
                            and_then_enter_item_like: None,
                        },
                    ));
                }
            };

            match item_like_type {
                ItemLikeType::BlockQuoteLine => {
                    nesting.processed_item_likes += 1;
                    if is_expecting_deeper {
                        stack.push(item_like_type.into());
                        nesting.item_likes_in_stack += 1;
                        return InternalResult::ToYield(
                            State::ExpectingContainer,
                            ToYield::One(BlockEvent::EnterBlockQuote),
                        );
                    } else {
                        return InternalResult::ToContinue(State::ExpectingContainer);
                    }
                }
                ItemLikeType::OrderedListItem | ItemLikeType::UnorderedListItem => {
                    if is_expecting_deeper {
                        let to_yield = enter_item_like(stack, nesting, item_like_type, true);
                        return InternalResult::ToYield(State::ExpectingContainer, to_yield);
                    } else {
                        let is_item_like_type_same_with_last_line = self
                            .get_nth_item_like_in_stack(stack, nesting.processed_item_likes)
                            == item_like_type;
                        return InternalResult::ToContinue(State::ExitingDiscontinuedItemLikes(
                            ExitingDiscontinuedItemLikesState {
                                should_keep_container: is_item_like_type_same_with_last_line,
                                and_then_enter_item_like: Some(item_like_type),
                            },
                        ));
                    }
                }
            }
        }

        InternalResult::ToContinue(State::ExpectingLeaf)
    }

    #[inline(always)]
    fn process_in_expecting_leaf_state(&mut self, ctx: &mut Context<'a>) -> InternalResult<'a> {
        // XXX: 由于在状态转移到 [State::ExpectingLeaf] 之前一定调用过
        // `ctx.scan_blank_text`，因此 `ctx.mapper.peek_1()` 一定不对应空白字符（
        // “空白字符” 不包含换行。）。

        let Some(peeked) = ctx.mapper.peek_1() else {
            return InternalResult::Done;
        };
        if peeked.is_line_feed() {
            let new_state = State::Start {
                is_certain_is_new_line: None,
            };
            return InternalResult::ToContinue(new_state);
        }

        match scan_leaf(ctx) {
            LeafType::ThematicBreak => {
                let new_state = State::Start {
                    is_certain_is_new_line: None,
                };
                InternalResult::ToYield(new_state, ToYield::One(BlockEvent::ThematicBreak))
            }
            LeafType::Heading { leading_signs } => InternalResult::ToSwitchToSubParser(Box::new(
                sub_parsers::heading::Parser::new(leading_signs),
            )),
            LeafType::CodeBlock { backticks } => InternalResult::ToSwitchToSubParser(Box::new(
                sub_parsers::code_block::Parser::new(backticks),
            )),
            LeafType::Paragraph { content_before } => InternalResult::ToSwitchToSubParser(
                Box::new(sub_parsers::paragraph::Parser::new(content_before)),
            ),
        }
    }

    #[inline(always)]
    fn process_in_exiting_discontinued_item_likes_state(
        &mut self,
        stack: &mut Vec<StackEntry>,
        nesting: &mut Nesting,
        state: ExitingDiscontinuedItemLikesState,
    ) -> InternalResult<'a> {
        if nesting.processed_item_likes == nesting.item_likes_in_stack {
            return InternalResult::ToContinue(State::ExpectingContainer);
        }

        if let Some(mut sub_parser) = self.paused_sub_parser.take() {
            nesting.is_exiting_discontinued_item_likes = Some(state);
            sub_parser.resume_from_pause_for_new_line_and_exit();
            return InternalResult::ToSwitchToSubParser(sub_parser);
        }

        if let StackEntry::ItemLike(item_like_type_to_exit) = stack.pop().unwrap() {
            nesting.item_likes_in_stack -= 1;
            if nesting.processed_item_likes == nesting.item_likes_in_stack {
                let should_exit_container =
                    item_like_type_to_exit.has_container() && !state.should_keep_container;
                if should_exit_container {
                    stack.pop().unwrap();
                }
                if let Some(item_like_type_to_enter) = state.and_then_enter_item_like {
                    let to_yield = enter_item_like(
                        stack,
                        nesting,
                        item_like_type_to_enter,
                        !state.should_keep_container,
                    );
                    self.deferred_to_yield = Some(to_yield);
                }
                if should_exit_container {
                    return InternalResult::ToYield(
                        State::ExitingDiscontinuedItemLikes(state),
                        ToYield::Two(BlockEvent::Exit, BlockEvent::Exit),
                    );
                }
            }
        }
        InternalResult::ToYield(
            State::ExitingDiscontinuedItemLikes(state),
            ToYield::One(BlockEvent::Exit),
        )
    }

    /// `n` 基于 0。
    fn get_nth_item_like_in_stack(&mut self, stack: &[StackEntry], n: usize) -> ItemLikeType {
        let memo = self
            .get_nth_item_like_in_stack_memo
            .take()
            .unwrap_or_default();

        let mut n_countdown = n - memo.last_n;

        for (delta_index, entry) in stack.iter().skip(memo.last_index).enumerate() {
            if let StackEntry::ItemLike(item_like_type) = entry {
                if n_countdown == 0 {
                    self.get_nth_item_like_in_stack_memo = Some(GetNthItemLikeInStackMemo {
                        last_n: n,
                        last_index: memo.last_index + delta_index,
                    });
                    return *item_like_type;
                }
                n_countdown -= 1;
            }
        }
        unreachable!()
    }
}

#[inline(always)]
fn scan_item_like(ctx: &mut Context) -> Option<ItemLikeType> {
    match ctx.peek_next_three_chars() {
        [Some(b'>'), ref second_char, ..] => {
            if check_is_indeed_item_like(ctx, second_char) {
                Some(ItemLikeType::BlockQuoteLine)
            } else {
                None
            }
        }
        [Some(b'#'), ref second_char, ..] => {
            if check_is_indeed_item_like(ctx, second_char) {
                Some(ItemLikeType::OrderedListItem)
            } else {
                None
            }
        }
        [Some(b'*'), ref second_char, ..] => {
            if check_is_indeed_item_like(ctx, second_char) {
                Some(ItemLikeType::UnorderedListItem)
            } else {
                None
            }
        }
        _ => None,
    }
}

fn check_is_indeed_item_like(ctx: &mut Context, second_char: &Option<u8>) -> bool {
    let is_indeed_item_like = second_char == &Some(b' ') || {
        let p = ctx.mapper.peek_2();
        p.is_none() || p.is_some_and(|p| p.is_line_feed())
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

fn scan_leaf(ctx: &mut Context) -> LeafType {
    match ctx.peek_next_three_chars() {
        [Some(b'-'), Some(b'-'), Some(b'-')] => {
            ctx.must_take_from_mapper_and_apply_to_cursor(3);
            ctx.drop_from_mapper_while_char(b'-');
            LeafType::ThematicBreak
        }
        [Some(b'='), ..] => {
            ctx.must_take_from_mapper_and_apply_to_cursor(1);
            let mut potential_opening_part = Range::new(ctx.cursor.value().unwrap(), 1);
            let dropped = ctx.drop_from_mapper_while_char_with_maximum(b'=', 5);
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
        [Some(b'`'), Some(b'`'), Some(b'`')] => {
            ctx.must_take_from_mapper_and_apply_to_cursor(3);
            let extra_count = ctx.drop_from_mapper_while_char(b'`');
            LeafType::CodeBlock {
                backticks: 3 + extra_count,
            }
        }
        _ => LeafType::Paragraph {
            content_before: None,
        },
    }
}

fn enter_item_like(
    stack: &mut Vec<StackEntry>,
    nesting: &mut Nesting,
    item_like_type: ItemLikeType,
    should_enter_container: bool,
) -> ToYield {
    if !matches!(
        item_like_type,
        ItemLikeType::OrderedListItem | ItemLikeType::UnorderedListItem
    ) {
        unreachable!()
    }

    nesting.processed_item_likes += 1;
    if should_enter_container {
        stack.push(StackEntry::Container);
    }
    stack.push(item_like_type.into());
    nesting.item_likes_in_stack += 1;

    if should_enter_container {
        ToYield::Two(
            match item_like_type {
                ItemLikeType::BlockQuoteLine => unreachable!(),
                ItemLikeType::OrderedListItem => BlockEvent::EnterOrderedList,
                ItemLikeType::UnorderedListItem => BlockEvent::EnterUnorderedList,
            },
            BlockEvent::EnterListItem,
        )
    } else {
        ToYield::One(BlockEvent::EnterListItem)
    }
}
