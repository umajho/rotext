use derivative::Derivative;

use super::{
    context::Context, global_mapper::Mapped, sub_parsers, utils::ArrayQueue, ItemLikeType, Nesting,
    StackEntry,
};
use crate::{
    common::Range,
    events::{BlockEvent, NewLine},
};

pub struct Parser<'a> {
    paused_sub_parser: Option<Box<dyn sub_parsers::SubParser<'a> + 'a>>,

    state: State,
    current_line_number: usize,
    is_new_line: bool,
    to_yield: ArrayQueue<4, BlockEvent>,

    get_nth_item_like_in_stack_memo: Option<GetNthItemLikeInStackMemo>,
}

#[derive(Debug)]
enum State {
    Start { new_line: Option<NewLine> },
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
    ToContinue(State),
    ToSwitchToSubParser(#[derivative(Debug = "ignore")] Box<dyn sub_parsers::SubParser<'a> + 'a>),
    Done,
}

impl<'a> Parser<'a> {
    pub fn new(
        new_line: Option<NewLine>,
        paused_sub_parser: Option<Box<dyn sub_parsers::SubParser<'a> + 'a>>,
    ) -> Self {
        Self {
            paused_sub_parser,
            state: State::Start { new_line },
            // 这里只是随便初始化一下，实际在 [State::Start] 中决定。
            current_line_number: 0,
            // 这里只是随便初始化一下，实际在 [State::Start] 中决定。
            is_new_line: false,
            to_yield: ArrayQueue::new(),
            get_nth_item_like_in_stack_memo: None,
        }
    }

    pub fn parse(
        &mut self,
        ctx: &mut Context<'a>,
        stack: &mut Vec<StackEntry>,
        nesting: &mut Nesting,
    ) -> Result<'a> {
        loop {
            if let Some(ev) = self.to_yield.pop_front() {
                return Result::ToYield(ev);
            }

            #[cfg(debug_assertions)]
            log::debug!("BLOCK->ROOT state={:?}", self.state);

            let internal_result = match std::mem::replace(&mut self.state, State::Invalid) {
                State::Start { new_line } => self.process_in_start_state(ctx, nesting, new_line),
                State::ExpectingContainer => {
                    self.process_in_expecting_container_state(ctx, stack, nesting)
                }
                State::ExpectingLeaf => self.process_in_expecting_leaf_state(ctx),
                State::ExitingDiscontinuedItemLikes(state) => {
                    self.process_in_exiting_discontinued_item_likes_state(stack, nesting, state)
                }
                State::Invalid => unreachable!(),
            };

            #[cfg(debug_assertions)]
            log::debug!("BLOCK->ROOT internal_result={:?}", internal_result);

            match internal_result {
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
        new_line: Option<NewLine>,
    ) -> InternalResult<'a> {
        if let Some(state) = nesting.is_exiting_discontinued_item_likes.take() {
            self.is_new_line = true;
            return InternalResult::ToContinue(State::ExitingDiscontinuedItemLikes(state));
        }

        if let Some(new_line) = new_line {
            self.is_new_line = true;
            self.current_line_number = new_line.line_number_after;
        } else if let Some(Mapped::NewLine(new_line)) = ctx.mapper.peek(0) {
            self.is_new_line = true;
            self.current_line_number = new_line.line_number_after;
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
                paused_sub_parser.resume_from_pause_for_new_line_and_continue(NewLine {
                    line_number_after: self.current_line_number,
                });
                return InternalResult::ToSwitchToSubParser(paused_sub_parser);
            }
        }

        _ = ctx.scan_blank_text();
        if self.is_new_line {
            match scan_paragraph_or_item_like(ctx) {
                ScanParagraphOrItemLikeResult::None => {
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
                }
                ScanParagraphOrItemLikeResult::BlockQuoteLine => {
                    nesting.processed_item_likes += 1;
                    if is_expecting_deeper {
                        stack.push(StackEntry::BlockQuote);
                        nesting.item_likes_in_stack += 1;
                        self.to_yield.push_back(BlockEvent::EnterBlockQuote);
                        return InternalResult::ToContinue(State::ExpectingContainer);
                    } else {
                        return InternalResult::ToContinue(State::ExpectingContainer);
                    }
                }
                ScanParagraphOrItemLikeResult::ItemLike(item_like_type) => {
                    if is_expecting_deeper {
                        self.enter_item_like(stack, nesting, item_like_type, true);
                        return InternalResult::ToContinue(State::ExpectingContainer);
                    } else {
                        let last_item_like_type = self.get_nth_item_like_or_paragraph_in_stack(
                            stack,
                            nesting.processed_item_likes,
                        );
                        return InternalResult::ToContinue(State::ExitingDiscontinuedItemLikes(
                            ExitingDiscontinuedItemLikesState {
                                should_keep_container: are_item_likes_in_same_group(
                                    item_like_type,
                                    last_item_like_type,
                                ),
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

        let Some(peeked) = ctx.mapper.peek(0) else {
            return InternalResult::Done;
        };
        if peeked.is_new_line() {
            let new_state = State::Start { new_line: None };
            return InternalResult::ToContinue(new_state);
        }

        match scan_leaf(ctx) {
            LeafType::ThematicBreak => {
                let new_state = State::Start { new_line: None };
                self.to_yield.push_back(BlockEvent::ThematicBreak);
                InternalResult::ToContinue(new_state)
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
            unreachable!();
        }

        if let Some(mut sub_parser) = self.paused_sub_parser.take() {
            nesting.is_exiting_discontinued_item_likes = Some(state);
            sub_parser.resume_from_pause_for_new_line_and_exit();
            return InternalResult::ToSwitchToSubParser(sub_parser);
        }

        let top = stack.pop().unwrap();
        if matches!(top, StackEntry::ItemLike(_) | StackEntry::BlockQuote) {
            nesting.item_likes_in_stack -= 1;
            if nesting.processed_item_likes == nesting.item_likes_in_stack {
                self.to_yield.push_back(BlockEvent::Exit);
                if matches!(top, StackEntry::ItemLike(_)) && !state.should_keep_container {
                    stack.pop().unwrap();
                    self.to_yield.push_back(BlockEvent::Exit);
                }
                if let Some(item_like_type_to_enter) = state.and_then_enter_item_like {
                    self.enter_item_like(
                        stack,
                        nesting,
                        item_like_type_to_enter,
                        !state.should_keep_container,
                    );
                }
                return InternalResult::ToContinue(State::ExpectingContainer);
            }
        }
        self.to_yield.push_back(BlockEvent::Exit);
        InternalResult::ToContinue(State::ExitingDiscontinuedItemLikes(state))
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
            if let StackEntry::ItemLike(item_like_type_) = entry {
                item_like_type = Some(*item_like_type_);
            } else if !matches!(entry, StackEntry::BlockQuote) {
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

    fn enter_item_like(
        &mut self,
        stack: &mut Vec<StackEntry>,
        nesting: &mut Nesting,
        item_like_type: ItemLikeType,
        should_enter_container: bool,
    ) {
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
            stack.push(StackEntry::Container);
        }
        stack.push(item_like_type.into());
        nesting.item_likes_in_stack += 1;

        if should_enter_container {
            self.to_yield.push_back(match item_like_type {
                ItemLikeType::OrderedListItem => BlockEvent::EnterOrderedList,
                ItemLikeType::UnorderedListItem => BlockEvent::EnterUnorderedList,
                ItemLikeType::DescriptionTerm | ItemLikeType::DescriptionDetails => {
                    BlockEvent::EnterDescriptionList
                }
            });
        }
        self.to_yield.push_back(match item_like_type {
            ItemLikeType::OrderedListItem | ItemLikeType::UnorderedListItem => {
                BlockEvent::EnterListItem
            }
            ItemLikeType::DescriptionTerm => BlockEvent::EnterDescriptionTerm,
            ItemLikeType::DescriptionDetails => BlockEvent::EnterDescriptionDetails,
        });
    }
}

enum ScanParagraphOrItemLikeResult {
    BlockQuoteLine,
    ItemLike(ItemLikeType),
    None,
}
#[inline(always)]
fn scan_paragraph_or_item_like(ctx: &mut Context) -> ScanParagraphOrItemLikeResult {
    match ctx.peek_next_three_chars() {
        [Some(b'>'), ref second_char, ..] if check_is_indeed_item_like(ctx, second_char) => {
            return ScanParagraphOrItemLikeResult::BlockQuoteLine;
        }
        [Some(b'#'), ref second_char, ..] if check_is_indeed_item_like(ctx, second_char) => {
            return ScanParagraphOrItemLikeResult::ItemLike(ItemLikeType::OrderedListItem);
        }
        [Some(b'*'), ref second_char, ..] if check_is_indeed_item_like(ctx, second_char) => {
            return ScanParagraphOrItemLikeResult::ItemLike(ItemLikeType::UnorderedListItem);
        }
        [Some(b';'), ref second_char, ..] if check_is_indeed_item_like(ctx, second_char) => {
            return ScanParagraphOrItemLikeResult::ItemLike(ItemLikeType::DescriptionTerm);
        }
        [Some(b':'), ref second_char, ..] if check_is_indeed_item_like(ctx, second_char) => {
            return ScanParagraphOrItemLikeResult::ItemLike(ItemLikeType::DescriptionDetails);
        }
        _ => {}
    };
    ScanParagraphOrItemLikeResult::None
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
