use super::{context::Context, sub_parsers, ItemLikeType, Nesting, StackEntry};
use crate::{common::Range, events::BlockEvent};

pub struct Parser<'a> {
    paused_sub_parser: Option<Box<dyn sub_parsers::SubParser<'a> + 'a>>,

    state: State,
    is_new_line: bool,
}

enum State {
    Start,
    ExpectingContainer,
    ExpectingLeaf,
    ExitingDiscountinuedItemLikes,

    Invalid,
}

pub enum Result<'a> {
    ToYield(BlockEvent),
    ToSwitchToSubParser(Box<dyn sub_parsers::SubParser<'a> + 'a>),
    Done,
}

enum InternalResult<'a> {
    ToYield(State, BlockEvent),
    ToContinue(State),
    ToSwitchToSubParser(Box<dyn sub_parsers::SubParser<'a> + 'a>),
    Done,
}

impl<'a> Parser<'a> {
    pub fn new(paused_sub_parser: Option<Box<dyn sub_parsers::SubParser<'a> + 'a>>) -> Self {
        Self {
            paused_sub_parser,
            state: State::Start,
            // 这里只是随便初始化一下，实际在 [State::Start] 中决定。
            is_new_line: false,
        }
    }

    pub fn parse(
        &mut self,
        ctx: &mut Context<'a>,
        stack: &mut Vec<StackEntry>,
        nesting: &mut Nesting,
    ) -> Result<'a> {
        loop {
            let internal_result = match self.state {
                State::Start => self.process_in_start_state(ctx, nesting),
                State::ExpectingContainer => {
                    self.process_in_expecting_container_state(ctx, nesting)
                }
                State::ExpectingLeaf => self.process_in_expecting_leaf_state(ctx),
                State::ExitingDiscountinuedItemLikes => {
                    self.process_in_exiting_discountinued_item_likes_state(stack, nesting)
                }
                State::Invalid => unreachable!(),
            };

            match internal_result {
                InternalResult::ToYield(state, ev) => {
                    self.state = state;
                    break Result::ToYield(ev);
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
    ) -> InternalResult<'a> {
        if nesting.is_exiting_discountinued_item_likes {
            nesting.is_exiting_discountinued_item_likes = false;
            return InternalResult::ToContinue(State::ExitingDiscountinuedItemLikes);
        }

        self.is_new_line = ctx.mapper.peek_1().is_some_and(|p| p.is_line_feed());
        if self.is_new_line {
            ctx.must_take_from_mapper_and_apply_to_cursor(1);
            nesting.processed_item_likes = 0;
        }

        InternalResult::ToContinue(State::ExpectingContainer)
    }

    #[inline(always)]
    fn process_in_expecting_container_state(
        &mut self,
        ctx: &mut Context<'a>,
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
            let Some(item_like) = scan_item_like(ctx) else {
                if is_expecting_deeper {
                    return InternalResult::ToContinue(State::ExpectingLeaf);
                } else {
                    return InternalResult::ToContinue(State::ExitingDiscountinuedItemLikes);
                }
            };

            nesting.processed_item_likes += 1;
            match item_like {
                ItemLikeType::BlockQuoteLine => {
                    if is_expecting_deeper {
                        return InternalResult::ToYield(
                            State::ExpectingContainer,
                            BlockEvent::EnterBlockQuote,
                        );
                    } else {
                        return InternalResult::ToContinue(State::ExpectingContainer);
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
            return InternalResult::ToContinue(State::Start);
        }

        match scan_leaf(ctx) {
            LeafType::ThematicBreak => {
                InternalResult::ToYield(State::Start, BlockEvent::ThematicBreak)
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
    fn process_in_exiting_discountinued_item_likes_state(
        &mut self,
        stack: &mut Vec<StackEntry>,
        nesting: &mut Nesting,
    ) -> InternalResult<'a> {
        if nesting.processed_item_likes == nesting.item_likes_in_stack {
            return InternalResult::ToContinue(State::ExpectingContainer);
        }

        if let Some(mut sub_parser) = self.paused_sub_parser.take() {
            nesting.is_exiting_discountinued_item_likes = true;
            sub_parser.resume_from_pause_for_new_line_and_exit();
            return InternalResult::ToSwitchToSubParser(sub_parser);
        }

        if stack.pop().unwrap().is_item_like() {
            nesting.item_likes_in_stack -= 1;
        }

        InternalResult::ToYield(State::ExitingDiscountinuedItemLikes, BlockEvent::Exit)
    }
}

#[inline(always)]
fn scan_item_like(ctx: &mut Context) -> Option<ItemLikeType> {
    match ctx.peek_next_three_chars() {
        [Some(b'>'), second_char, ..] => {
            let is_indeed_item_like = second_char == Some(b' ') || {
                let p = ctx.mapper.peek_2();
                p.is_none() || p.is_some_and(|p| p.is_line_feed())
            };
            if is_indeed_item_like {
                let to_take = if second_char.is_some() { 2 } else { 1 };
                ctx.must_take_from_mapper_and_apply_to_cursor(to_take);
                Some(ItemLikeType::BlockQuoteLine)
            } else {
                None
            }
        }
        _ => None,
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
