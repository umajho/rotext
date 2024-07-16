mod context;
mod events;
mod global_mapper;
mod sub_parsers;
mod utils;

use context::Context;
pub use events::Event;

use crate::global;
use global_mapper::GlobalEventStreamMapper;
use utils::Peekable3;

pub struct Parser<'a, I: 'a + Iterator<Item = global::Event>> {
    /// 如果位 true，代表没有后续输入了，要清理栈中余留的内容。
    is_cleaning_up: bool,
    state: State<'a, I>,
    stack: Vec<StackEntry>,
}

enum State<'a, I: 'a + Iterator<Item = global::Event>> {
    InRoot(Box<Context<'a, I>>),
    InSubParser(Box<dyn sub_parsers::SubParser<'a, I> + 'a>),

    Invalid,
}

enum StackEntry {}

impl<'a, I: 'a + Iterator<Item = global::Event>> Parser<'a, I> {
    pub fn new(input: &'a [u8], global_stream: I) -> Parser<'a, I> {
        let ctx = Context {
            input,
            mapper: Peekable3::new(GlobalEventStreamMapper::new(input, global_stream)),
            cursor: utils::InputCursor::new(),
        };

        Parser {
            is_cleaning_up: false,
            state: State::InRoot(Box::new(ctx)),
            stack: vec![],
        }
    }

    pub fn next(&mut self) -> Option<Event> {
        loop {
            if self.is_cleaning_up {
                // 若栈中还有内容，出栈并返回 `Some(Event::Exit)`；若栈已空，返回
                // `None`。
                return self.stack.pop().map(|_| Event::Exit);
            }

            let to_break: Option<Event>;

            let state = std::mem::replace(&mut self.state, State::Invalid);
            (to_break, self.state) = match state {
                State::InRoot(ctx) => match parse_root(ctx) {
                    RootParseResult::ToYield(ctx, ev) => (Some(ev), State::InRoot(ctx)),
                    RootParseResult::ToEnter(sub_parser) => (None, State::InSubParser(sub_parser)),
                    RootParseResult::Done => {
                        self.is_cleaning_up = true;
                        (None, State::Invalid)
                    }
                },
                State::InSubParser(mut sub_parser) => {
                    let ev = sub_parser.next();
                    if ev.is_none() {
                        (None, State::InRoot(sub_parser.take_context()))
                    } else {
                        (ev, State::InSubParser(sub_parser))
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

impl<'a, I: 'a + Iterator<Item = global::Event>> Iterator for Parser<'a, I> {
    type Item = Event;

    fn next(&mut self) -> Option<Self::Item> {
        self.next()
    }
}

enum RootParseResult<'a, I: 'a + Iterator<Item = global::Event>> {
    ToYield(Box<Context<'a, I>>, Event),
    ToEnter(Box<dyn sub_parsers::SubParser<'a, I> + 'a>),
    Done,
}

fn parse_root<'a, I: 'a + Iterator<Item = global::Event>>(
    mut ctx: Box<Context<'a, I>>,
) -> RootParseResult<'a, I> {
    loop {
        let Some(peeked) = ctx.mapper.peek_1() else {
            return RootParseResult::Done;
        };

        match peeked {
            global_mapper::Mapped::LineFeed
            | global_mapper::Mapped::BlankLine { .. }
            | global_mapper::Mapped::SpacesAtLineBeginning(_) => {
                ctx.mapper.next();
                continue;
            }
            global_mapper::Mapped::Text(_) => {
                return RootParseResult::ToEnter(Box::new(sub_parsers::inline::Parser::new(ctx)));
            }
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
            RootParseResult::ToYield(ctx, Event::ThematicBreak)
        }
        [Some(b'`'), Some(b'`'), Some(b'`')] => {
            ctx.must_take_from_mapper_and_apply_to_cursor(3);
            todo!()
        }
        _ => RootParseResult::ToEnter(Box::new(sub_parsers::inline::Parser::new(ctx))),
    }
}

fn is_space_char(char: u8) -> bool {
    char == b' ' || char == b'\t'
}
