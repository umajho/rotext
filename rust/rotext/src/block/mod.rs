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
}

enum State<'a> {
    InRoot,
    InRootWithPausedSubParser(Box<dyn sub_parsers::SubParser<'a> + 'a>),
    InSubParser(Box<dyn sub_parsers::SubParser<'a> + 'a>),

    Invalid,
}

enum StackEntry {}

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
            state: State::InRoot,
            stack: vec![],
        }
    }

    pub fn next(&mut self) -> Option<BlockEvent> {
        loop {
            if self.is_cleaning_up {
                // 若栈中还有内容，出栈并返回 `Some(Event::Exit)`；若栈已空，返回
                // `None`。
                return self.stack.pop().map(|_| BlockEvent::Exit);
            }

            let to_break: Option<BlockEvent>;

            let state = std::mem::replace(&mut self.state, State::Invalid);
            (to_break, self.state) = match state {
                State::InRoot => match parse_root(&mut self.context) {
                    RootParseResult::ToYield(ev) => (Some(ev), State::InRoot),
                    RootParseResult::ToEnter(sub_parser) => (None, State::InSubParser(sub_parser)),
                    RootParseResult::ToEnterParagraph => (
                        None,
                        State::InSubParser(Box::new(sub_parsers::paragraph::Parser::new(None))),
                    ),
                    RootParseResult::ToEnterParagraphWithContentBefore(content_before) => (
                        None,
                        State::InSubParser(Box::new(sub_parsers::paragraph::Parser::new(Some(
                            content_before,
                        )))),
                    ),
                    RootParseResult::Done => {
                        self.is_cleaning_up = true;
                        (None, State::Invalid)
                    }
                },
                State::InRootWithPausedSubParser(mut sub_parser) => {
                    sub_parser.resume_from_pause_for_new_line_and_continue();
                    (None, State::InSubParser(sub_parser))
                }
                State::InSubParser(mut sub_parser) => {
                    let result = sub_parser.next(&mut self.context);
                    match result {
                        sub_parsers::Result::ToYield(ev) => {
                            (Some(ev), State::InSubParser(sub_parser))
                        }
                        sub_parsers::Result::ToPauseForNewLine => {
                            (None, State::InRootWithPausedSubParser(sub_parser))
                        }
                        sub_parsers::Result::Done => (None, State::InRoot),
                    }
                }
                State::Invalid => unreachable!(),
            };

            if to_break.is_some() {
                break to_break;
            }
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
    ToYield(BlockEvent),
    ToEnter(Box<dyn sub_parsers::SubParser<'a> + 'a>),
    ToEnterParagraph,
    ToEnterParagraphWithContentBefore(Range),
    Done,
}

fn parse_root<'a>(ctx: &mut Context<'a>) -> RootParseResult<'a> {
    loop {
        let Some(peeked) = ctx.mapper.peek_1() else {
            return RootParseResult::Done;
        };

        match peeked {
            global_mapper::Mapped::LineFeed | global_mapper::Mapped::BlankAtLineBeginning(_) => {
                ctx.mapper.next();
                continue;
            }
            global_mapper::Mapped::Text(_) => return RootParseResult::ToEnterParagraph,
            global_mapper::Mapped::CharAt(_) | global_mapper::Mapped::NextChar => {
                if !ctx.take_from_mapper_and_apply_to_cursor_if_applied_cursor_satisfies(
                    |applied_cursor| applied_cursor.at(ctx.input).is_some_and(is_space_char),
                ) {
                    // peeked 所对应的字符不是空白字符。
                    break;
                }
            }
        }
    }

    match ctx.peek_next_three_chars() {
        [Some(b'-'), Some(b'-'), Some(b'-')] => {
            ctx.must_take_from_mapper_and_apply_to_cursor(3);
            ctx.drop_from_mapper_while_char(b'-');
            RootParseResult::ToYield(BlockEvent::ThematicBreak)
        }
        [Some(b'='), ..] => {
            ctx.must_take_from_mapper_and_apply_to_cursor(1);
            let mut potential_opening_part = Range::new(ctx.cursor.value().unwrap(), 1);
            let dropped = ctx.drop_from_mapper_while_char_with_maximum(b'=', 5);
            potential_opening_part.increase_length(dropped);

            if ctx.peek_next_char() == Some(b' ') {
                ctx.must_take_from_mapper_and_apply_to_cursor(1);
                RootParseResult::ToEnter(Box::new(sub_parsers::heading::Parser::new(1 + dropped)))
            } else {
                RootParseResult::ToEnterParagraphWithContentBefore(potential_opening_part)
            }
        }
        [Some(b'`'), Some(b'`'), Some(b'`')] => {
            ctx.must_take_from_mapper_and_apply_to_cursor(3);
            let extra_count = ctx.drop_from_mapper_while_char(b'`');
            RootParseResult::ToEnter(Box::new(sub_parsers::code_block::Parser::new(
                3 + extra_count,
            )))
        }
        _ => RootParseResult::ToEnterParagraph,
    }
}

fn is_space_char(char: u8) -> bool {
    char == b' ' || char == b'\t'
}
