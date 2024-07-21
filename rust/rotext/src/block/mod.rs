mod context;
mod global_mapper;
mod sub_parsers;
mod utils;

mod tests;

use context::Context;

use crate::{common::Range, events::BlockEvent, global};
use global_mapper::GlobalEventStreamMapper;
use utils::Peekable3;

pub struct Parser<'a> {
    context: Context<'a>,

    /// 如果为 true，代表没有后续输入了，要清理栈中余留的内容。
    is_cleaning_up: bool,
    state: State<'a>,
    stack: Vec<StackEntry>,

    /// 目前栈中有多少 item-likes。
    item_likes_in_stack: usize,
    /// 目前已处理了多少 item-likes。（每次换行后重置。）
    processed_item_likes_in_current_line: usize,
    /// 如果为 true，代表本行已经处理完 item-likes，在处理其他事件前，要将栈中
    /// item-likes 的数量缩减至与 [ItemLikesState::processed_count] 相同。
    is_item_likes_process_over_in_current_line: bool,
}

enum State<'a> {
    InRoot(StateInRoot<'a>),
    InSubParser(Box<dyn sub_parsers::SubParser<'a> + 'a>),

    Invalid,
}
struct StateInRoot<'a> {
    paused_sub_parser: Option<Box<dyn sub_parsers::SubParser<'a> + 'a>>,
}

enum StackEntry {
    ItemLike(ItemLikeType),
}
impl From<ItemLikeType> for StackEntry {
    fn from(value: ItemLikeType) -> Self {
        Self::ItemLike(value)
    }
}
impl StackEntry {
    pub fn is_item_like(&self) -> bool {
        matches!(self, StackEntry::ItemLike(_))
    }
}

enum ItemLikeType {
    BlockQuoteLine,
}
impl From<u8> for ItemLikeType {
    fn from(value: u8) -> Self {
        match value {
            b'>' => ItemLikeType::BlockQuoteLine,
            _ => unreachable!(),
        }
    }
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a [u8], global_stream: global::Parser<'a>) -> Parser<'a> {
        let context = Context {
            input,
            mapper: Peekable3::new(GlobalEventStreamMapper::new(input, global_stream)),
            cursor: utils::InputCursor::new(),
        };

        Parser {
            context,

            is_cleaning_up: false,
            state: State::InRoot(StateInRoot {
                paused_sub_parser: None,
            }),
            stack: vec![],
            item_likes_in_stack: 0,
            processed_item_likes_in_current_line: 0,
            is_item_likes_process_over_in_current_line: false,
        }
    }

    pub fn next(&mut self) -> Option<BlockEvent> {
        loop {
            if self.is_cleaning_up {
                // 若栈中还有内容，出栈并返回 `Some(Event::Exit)`；若栈已空，返
                // 回 `None`。
                return self.stack.pop().map(|_| BlockEvent::Exit);
            }

            let to_break: Option<BlockEvent>;
            (to_break, self.state) = match std::mem::replace(&mut self.state, State::Invalid) {
                State::InRoot(state) => self.process_in_root_state(state),
                State::InSubParser(sub_parser) => self.process_in_sub_parser_state(sub_parser),
                State::Invalid => unreachable!(),
            };

            if to_break.is_some() {
                break to_break;
            }
        }
    }

    #[inline(always)]
    fn process_in_root_state(&mut self, state: StateInRoot<'a>) -> (Option<BlockEvent>, State<'a>) {
        match self.parse_root(state.paused_sub_parser) {
            RootParseResult::ToYield(paused_sub_parser, ev) => {
                let new_state = State::InRoot(StateInRoot { paused_sub_parser });
                (Some(ev), new_state)
            }
            RootParseResult::ToEnter(sub_parser) => (None, State::InSubParser(sub_parser)),
            RootParseResult::ToEnterParagraph => {
                let p_parser = Box::new(sub_parsers::paragraph::Parser::new(None));
                (None, State::InSubParser(p_parser))
            }
            RootParseResult::ToEnterParagraphWithContentBefore(content_before) => {
                let p_parser = Box::new(sub_parsers::paragraph::Parser::new(Some(content_before)));
                (None, State::InSubParser(p_parser))
            }
            RootParseResult::ToContinue => {
                let new_state = State::InRoot(StateInRoot {
                    paused_sub_parser: None,
                });
                (None, new_state)
            }
            RootParseResult::Done => {
                self.is_cleaning_up = true;
                (None, State::Invalid)
            }
        }
    }

    #[inline(always)]
    fn process_in_sub_parser_state(
        &mut self,
        mut sub_parser: Box<dyn sub_parsers::SubParser<'a> + 'a>,
    ) -> (Option<BlockEvent>, State<'a>) {
        match sub_parser.next(&mut self.context) {
            sub_parsers::Result::ToYield(ev) => (Some(ev), State::InSubParser(sub_parser)),
            sub_parsers::Result::ToPauseForNewLine => {
                let new_state = State::InRoot(StateInRoot {
                    paused_sub_parser: Some(sub_parser),
                });
                (None, new_state)
            }
            sub_parsers::Result::Done => {
                let new_state = State::InRoot(StateInRoot {
                    paused_sub_parser: None,
                });
                (None, new_state)
            }
        }
    }

    fn parse_root(
        &mut self,
        mut paused_sub_parser: Option<Box<dyn sub_parsers::SubParser<'a> + 'a>>,
    ) -> RootParseResult<'a> {
        // 在某一行确定了会延续上一行中多少层 item-likes 之后，将会在这里依次
        // exit 掉层数更深的那些 item likes。
        if self.is_item_likes_process_over_in_current_line
            && self.item_likes_in_stack > self.processed_item_likes_in_current_line
        {
            let popped = self.stack.pop().unwrap();
            if popped.is_item_like() {
                self.item_likes_in_stack -= 1;
            }
            return RootParseResult::ToYield(paused_sub_parser, BlockEvent::Exit);
        }
        self.is_item_likes_process_over_in_current_line = false;

        if let Some(mut paused_sub_parser_) = paused_sub_parser.take() {
            let Some(peeked) = self.context.mapper.peek_1() else {
                paused_sub_parser_.resume_from_pause_for_new_line_and_exit();
                return RootParseResult::ToEnter(paused_sub_parser_);
            };

            if peeked.is_line_feed() {
                // 新的一行。

                self.context.must_take_from_mapper_and_apply_to_cursor(1);
                // 清空 item likes 之前针对单行记录的状态。
                self.processed_item_likes_in_current_line = 0;
                self.is_item_likes_process_over_in_current_line = true;

                fn is_next_blank_line_and_should_consume_one(
                    mapper: &mut Peekable3<GlobalEventStreamMapper>,
                ) -> (bool, bool) {
                    if mapper.peek_1().is_some_and(|p| p.is_line_feed()) {
                        (true, false)
                    } else if mapper
                        .peek_1()
                        .is_some_and(|p| p.is_blank_at_line_beginning())
                        && mapper.peek_2().is_some_and(|p| p.is_line_feed())
                    {
                        (true, true)
                    } else {
                        (false, true)
                    }
                }

                let (is_next_blank_line, should_consume_one) =
                    is_next_blank_line_and_should_consume_one(&mut self.context.mapper);

                if is_next_blank_line {
                    if should_consume_one {
                        self.context.must_take_from_mapper_and_apply_to_cursor(1);
                    }
                    paused_sub_parser_.resume_from_pause_for_new_line_and_continue();
                    return RootParseResult::ToEnter(paused_sub_parser_);
                }
            }

            paused_sub_parser = Some(paused_sub_parser_);
        } else {
            // 没有暂停的子解析器时，直到非空白内容前的空白都可以无视掉。有子解析器时，由于要解析
            // 的内容可能是逐字内容，不能无视空白，因此不能这么做。
            loop {
                let Some(peeked) = self.context.mapper.peek_1() else {
                    return RootParseResult::Done;
                };
                match peeked {
                    global_mapper::Mapped::LineFeed => {
                        self.context.must_take_from_mapper_and_apply_to_cursor(1);
                        // 清空 item likes 之前针对单行记录的状态。
                        self.processed_item_likes_in_current_line = 0;
                        self.is_item_likes_process_over_in_current_line = true;
                    }
                    global_mapper::Mapped::BlankAtLineBeginning(_) => {
                        self.context.must_take_from_mapper_and_apply_to_cursor(1);
                    }
                    m if self.context.cursor.applying(m).at(self.context.input) == Some(b' ') => {
                        self.context.must_take_from_mapper_and_apply_to_cursor(1);
                    }
                    _ => break,
                }
            }
        }

        if self.context.mapper.peek_1().is_none() {
            if let Some(mut paused_sub_parser) = paused_sub_parser {
                paused_sub_parser.resume_from_pause_for_new_line_and_exit();
                return RootParseResult::ToEnter(paused_sub_parser);
            }
            return RootParseResult::Done;
        };

        let next_three_chars: [Option<u8>; 3];
        'PROCESSING_ITEM_LIKES: {
            if self.is_item_likes_process_over_in_current_line {
                // 解析 item-likes 的阶段已经结束了，所以直接 break 出去。
                next_three_chars = self.context.peek_next_three_chars();
                break 'PROCESSING_ITEM_LIKES;
            }
            if self.processed_item_likes_in_current_line < self.item_likes_in_stack {
                unreachable!()
            }

            // 已经处理过的 item-likes 和在栈中的相同，代表接下来是尝试找到更深的 item-likes。
            let is_to_find_deeper =
                self.processed_item_likes_in_current_line == self.item_likes_in_stack;
            if is_to_find_deeper {
                if let Some(mut paused_sub_parser) = paused_sub_parser {
                    // 更深的位置已经被子解析器处理的内容占用了。由于已经处理过 item-likes 的
                    // 和在栈中的相同，不用 exit，可以直接交给子解析器。
                    paused_sub_parser.resume_from_pause_for_new_line_and_continue();
                    return RootParseResult::ToEnter(paused_sub_parser);
                }
            }

            next_three_chars = self.context.peek_next_three_chars();
            let Some(item_like_type) = try_parse_item_like(&mut self.context, &next_three_chars)
            else {
                break 'PROCESSING_ITEM_LIKES;
            };

            match item_like_type {
                ItemLikeType::BlockQuoteLine => {
                    self.processed_item_likes_in_current_line += 1;
                    if is_to_find_deeper {
                        self.item_likes_in_stack += 1;
                        self.stack.push(item_like_type.into());
                        return RootParseResult::ToYield(None, BlockEvent::EnterBlockQuote);
                    } else {
                        return RootParseResult::ToContinue;
                    }
                }
            }
        }

        if self.processed_item_likes_in_current_line < self.item_likes_in_stack {
            self.is_item_likes_process_over_in_current_line = true;
            if let Some(mut paused_sub_parser) = paused_sub_parser {
                paused_sub_parser.resume_from_pause_for_new_line_and_exit();
                return RootParseResult::ToEnter(paused_sub_parser);
            } else {
                return RootParseResult::ToContinue;
            }
        } else if let Some(mut paused_sub_parser) = paused_sub_parser {
            paused_sub_parser.resume_from_pause_for_new_line_and_continue();
            return RootParseResult::ToEnter(paused_sub_parser);
        }

        match next_three_chars {
            [Some(b'-'), Some(b'-'), Some(b'-')] => {
                self.context.must_take_from_mapper_and_apply_to_cursor(3);
                self.context.drop_from_mapper_while_char(b'-');
                RootParseResult::ToYield(None, BlockEvent::ThematicBreak)
            }
            [Some(b'='), ..] => {
                self.context.must_take_from_mapper_and_apply_to_cursor(1);
                let mut potential_opening_part =
                    Range::new(self.context.cursor.value().unwrap(), 1);
                let dropped = self
                    .context
                    .drop_from_mapper_while_char_with_maximum(b'=', 5);
                potential_opening_part.increase_length(dropped);

                if self.context.peek_next_char() == Some(b' ') {
                    self.context.must_take_from_mapper_and_apply_to_cursor(1);
                    RootParseResult::ToEnter(Box::new(sub_parsers::heading::Parser::new(
                        1 + dropped,
                    )))
                } else {
                    RootParseResult::ToEnterParagraphWithContentBefore(potential_opening_part)
                }
            }
            [Some(b'`'), Some(b'`'), Some(b'`')] => {
                self.context.must_take_from_mapper_and_apply_to_cursor(3);
                let extra_count = self.context.drop_from_mapper_while_char(b'`');
                RootParseResult::ToEnter(Box::new(sub_parsers::code_block::Parser::new(
                    3 + extra_count,
                )))
            }
            _ => RootParseResult::ToEnterParagraph,
        }
    }
}

impl<'a> Iterator for Parser<'a> {
    type Item = BlockEvent;

    fn next(&mut self) -> Option<Self::Item> {
        self.next()
    }
}

enum RootParseResult<'a> {
    ToYield(Option<Box<dyn sub_parsers::SubParser<'a> + 'a>>, BlockEvent),
    ToEnter(Box<dyn sub_parsers::SubParser<'a> + 'a>),
    ToEnterParagraph,
    ToEnterParagraphWithContentBefore(Range),
    ToContinue,
    Done,
}

#[inline(always)]
fn try_parse_item_like(ctx: &mut Context, peeked: &[Option<u8>; 3]) -> Option<ItemLikeType> {
    match peeked {
        [Some(b'>'), second_char, ..] => {
            let is_indeed_item_like = second_char == &Some(b' ') || {
                let p = ctx.mapper.peek_2();
                p.is_none() || p.is_some_and(|p| p.is_line_feed())
            };
            if is_indeed_item_like {
                let to_take = if second_char.is_some() { 2 } else { 1 };
                ctx.must_take_from_mapper_and_apply_to_cursor(to_take);
                todo!()
            } else {
                None
            }
        }
        _ => None,
    }
}
