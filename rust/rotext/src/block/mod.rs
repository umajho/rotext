mod context;
mod events;
mod global_mapper;
mod sub_parsers;
mod utils;

use context::Context;
pub use events::Event;

use crate::{common::Range, global};
use global_mapper::GlobalEventStreamMapper;
use utils::Peekable3;

pub struct Parser<'a, I: 'a + Iterator<Item = global::Event>> {
    context: Context<'a, I>,

    /// 如果为 true，代表没有后续输入了，要清理栈中余留的内容。
    is_cleaning_up: bool,
    state: State<'a, I>,
    stack: Vec<StackEntry>,
}

enum State<'a, I: 'a + Iterator<Item = global::Event>> {
    InRoot,
    InRootWithPausedSubParser(Box<dyn sub_parsers::SubParser<'a, I> + 'a>),
    InSubParser(Box<dyn sub_parsers::SubParser<'a, I> + 'a>),

    Invalid,
}

enum StackEntry {}

impl<'a, I: 'a + Iterator<Item = global::Event>> Parser<'a, I> {
    pub fn new(input: &'a [u8], global_stream: I) -> Parser<'a, I> {
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
                State::InRoot => match parse_root(&mut self.context) {
                    RootParseResult::ToYield(ev) => (Some(ev), State::InRoot),
                    RootParseResult::ToEnter(sub_parser) => (None, State::InSubParser(sub_parser)),
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

impl<'a, I: 'a + Iterator<Item = global::Event>> Iterator for Parser<'a, I> {
    type Item = Event;

    fn next(&mut self) -> Option<Self::Item> {
        self.next()
    }
}

enum RootParseResult<'a, I: 'a + Iterator<Item = global::Event>> {
    ToYield(Event),
    ToEnter(Box<dyn sub_parsers::SubParser<'a, I> + 'a>),
    Done,
}

fn parse_root<'a, I: 'a + Iterator<Item = global::Event>>(
    ctx: &mut Context<'a, I>,
) -> RootParseResult<'a, I> {
    loop {
        let Some(peeked) = ctx.mapper.peek_1() else {
            return RootParseResult::Done;
        };

        match peeked {
            global_mapper::Mapped::LineFeed | global_mapper::Mapped::BlankAtLineBeginning(_) => {
                ctx.mapper.next();
                continue;
            }
            global_mapper::Mapped::Text(_) => {
                return RootParseResult::ToEnter(Box::new(sub_parsers::paragraph::Parser::new(
                    None,
                )));
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
            RootParseResult::ToYield(Event::ThematicBreak)
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
                RootParseResult::ToEnter(Box::new(sub_parsers::paragraph::Parser::new(Some(
                    potential_opening_part,
                ))))
            }
        }
        [Some(b'`'), Some(b'`'), Some(b'`')] => {
            ctx.must_take_from_mapper_and_apply_to_cursor(3);
            let extra_count = ctx.drop_from_mapper_while_char(b'`');
            RootParseResult::ToEnter(Box::new(sub_parsers::code_block::Parser::new(
                3 + extra_count,
            )))
        }
        _ => RootParseResult::ToEnter(Box::new(sub_parsers::paragraph::Parser::new(None))),
    }
}

fn is_space_char(char: u8) -> bool {
    char == b' ' || char == b'\t'
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    use crate::events::EventType;

    type EventCase<'a> = (EventType, Option<&'a str>);

    #[rstest]
    // ## 空
    #[case(vec![""], vec![])]
    // ## 段落
    #[case(vec!["a", " a", "\na", "a\n"], vec![
        (EventType::EnterParagraph, None),
        (EventType::Unparsed, Some("a")),
        (EventType::Exit, None)])]
    #[case(vec!["a "], vec![
        (EventType::EnterParagraph, None),
        (EventType::Unparsed, Some("a ")),
        (EventType::Exit, None)])]
    #[case(vec!["a\nb", "a\n b"], vec![
        (EventType::EnterParagraph, None),
        (EventType::Unparsed, Some("a")),
        (EventType::LineFeed, None),
        (EventType::Unparsed, Some("b")),
        (EventType::Exit, None)])]
    #[case(vec!["a\n\nb", "a\n\n b"], vec![
        (EventType::EnterParagraph, None),
        (EventType::Unparsed, Some("a")),
        (EventType::Exit, None),
        (EventType::EnterParagraph, None),
        (EventType::Unparsed, Some("b")),
        (EventType::Exit, None)])]
    // ### 段落与全局阶段语法的互动
    #[case(vec!["a<`c`>"], vec![
        (EventType::EnterParagraph, None),
        (EventType::Unparsed, Some("a")),
        (EventType::Text, Some("c")),
        (EventType::Exit, None)])]
    #[case(vec!["<`c`>b"], vec![
        (EventType::EnterParagraph, None),
        (EventType::Text, Some("c")),
        (EventType::Unparsed, Some("b")),
        (EventType::Exit, None)])]
    #[case(vec!["a<`c`>b"], vec![
        (EventType::EnterParagraph, None),
        (EventType::Unparsed, Some("a")),
        (EventType::Text, Some("c")),
        (EventType::Unparsed, Some("b")),
        (EventType::Exit, None)])]
    #[case(vec!["a\n<`c`>", "a\n <`c`>"], vec![
        (EventType::EnterParagraph, None),
        (EventType::Unparsed, Some("a")),
        (EventType::LineFeed, None),
        (EventType::Text, Some("c")),
        (EventType::Exit, None)])]
    // ### “继续段落” 的优先级 “高于开启其他块级语法” 的优先级
    #[case(vec!["a\n---"], vec![ // 分割线
        (EventType::EnterParagraph, None),
        (EventType::Unparsed, Some("a")),
        (EventType::LineFeed, None),
        (EventType::Unparsed, Some("---")),
        (EventType::Exit, None)])]
    // ## 分割线
    #[case(vec!["---", "----"], vec![
        (EventType::ThematicBreak, None)])]
    #[case(vec!["--"], vec![
        (EventType::EnterParagraph, None),
        (EventType::Unparsed, Some("--")),
        (EventType::Exit, None)])]
    #[case(vec!["---\n---", "--- ---"], vec![
        (EventType::ThematicBreak, None),
        (EventType::ThematicBreak, None)])]
    #[case(vec!["---\na", "---a", "--- a"], vec![
        (EventType::ThematicBreak, None),
        (EventType::EnterParagraph, None),
        (EventType::Unparsed, Some("a")),
        (EventType::Exit, None)])]
    // ### 分割线与全局阶段语法的互动
    #[case(vec!["---\n<`a`>", "---<`a`>", "--- <`a`>"], vec![
        (EventType::ThematicBreak, None),
        (EventType::EnterParagraph, None),
        (EventType::Text, Some("a")),
        (EventType::Exit, None)])]
    // ## 段落
    #[case(vec!["= a ="], vec![
        (EventType::EnterHeading1, None),
        (EventType::Unparsed, Some("a")),
        (EventType::Exit, None)])]
    #[case(vec!["== a ==", "== a ==\n", "== a ==\n\n"], vec![
        (EventType::EnterHeading2, None),
        (EventType::Unparsed, Some("a")),
        (EventType::Exit, None)])]
    #[case(vec!["=== a ==="], vec![
        (EventType::EnterHeading3, None),
        (EventType::Unparsed, Some("a")),
        (EventType::Exit, None)])]
    #[case(vec!["==== a ===="], vec![
        (EventType::EnterHeading4, None),
        (EventType::Unparsed, Some("a")),
        (EventType::Exit, None)])]
    #[case(vec!["===== a ====="], vec![
        (EventType::EnterHeading5, None),
        (EventType::Unparsed, Some("a")),
        (EventType::Exit, None)])]
    #[case(vec!["====== a ======"], vec![
        (EventType::EnterHeading6, None),
        (EventType::Unparsed, Some("a")),
        (EventType::Exit, None)])]
    #[case(vec!["== a"], vec![
        (EventType::EnterHeading2, None),
        (EventType::Unparsed, Some("a")),
        (EventType::Exit, None)])]
    #[case(vec!["== a ="], vec![
        (EventType::EnterHeading2, None),
        (EventType::Unparsed, Some("a =")),
        (EventType::Exit, None)])]
    #[case(vec!["== a ==="], vec![
        (EventType::EnterHeading2, None),
        (EventType::Unparsed, Some("a ===")),
        (EventType::Exit, None)])]
    #[case(vec!["==  a  =="], vec![
        (EventType::EnterHeading2, None),
        (EventType::Unparsed, Some(" a ")),
        (EventType::Exit, None)])]
    #[case(vec!["== a ==\nb", "== a ==\n\nb"], vec![
        (EventType::EnterHeading2, None),
        (EventType::Unparsed, Some("a")),
        (EventType::Exit, None),
        (EventType::EnterParagraph, None),
        (EventType::Unparsed, Some("b")),
        (EventType::Exit, None)])]
    #[case(vec!["== a ==\n=== b ===", "== a ==\n\n=== b ==="], vec![
        (EventType::EnterHeading2, None),
        (EventType::Unparsed, Some("a")),
        (EventType::Exit, None),
        (EventType::EnterHeading3, None),
        (EventType::Unparsed, Some("b")),
        (EventType::Exit, None)])]
    #[case(vec!["==a =="], vec![
        (EventType::EnterParagraph, None),
        (EventType::Unparsed, Some("==a ==")),
        (EventType::Exit, None)])]
    #[case(vec!["== a=="], vec![
        (EventType::EnterHeading2, None),
        (EventType::Unparsed, Some("a==")),
        (EventType::Exit, None)])]
    #[case(vec!["== a == "], vec![
        (EventType::EnterHeading2, None),
        (EventType::Unparsed, Some("a == ")),
        (EventType::Exit, None)])]
    #[case(vec!["== a ==b"], vec![
        (EventType::EnterHeading2, None),
        (EventType::Unparsed, Some("a ==b")),
        (EventType::Exit, None)])]
    #[case(vec!["== a == b =="], vec![
        (EventType::EnterHeading2, None),
        (EventType::Unparsed, Some("a == b")),
        (EventType::Exit, None)])]
    #[case(vec!["======= a ======="], vec![
        (EventType::EnterParagraph, None),
        (EventType::Unparsed, Some("======= a =======")),
        (EventType::Exit, None)])]
    #[case(vec!["== <`c`> =="], vec![
        (EventType::EnterHeading2, None),
        (EventType::Text, Some("c")),
        (EventType::Exit, None)])]
    #[case(vec!["== a<`c`>b =="], vec![
        (EventType::EnterHeading2, None),
        (EventType::Unparsed, Some("a")),
        (EventType::Text, Some("c")),
        (EventType::Unparsed, Some("b")),
        (EventType::Exit, None)])]
    // ## 代码块
    #[case(vec!["```\ncode\n```", "```\ncode\n````", "````\ncode\n````"], vec![
        (EventType::EnterCodeBlock, None),
        (EventType::Separator, None),
        (EventType::Text, Some("code")),
        (EventType::Exit, None)])]
    #[case(vec!["```\n```", "```\n````", "````\n````"], vec![
        (EventType::EnterCodeBlock, None),
        (EventType::Separator, None),
        (EventType::Exit, None)])]
    #[case(vec!["```\n\n```"], vec![
        (EventType::EnterCodeBlock, None),
        (EventType::Separator, None),
        (EventType::LineFeed, None),
        (EventType::Exit, None)])]
    #[case(vec!["````\n```\n````"], vec![
        (EventType::EnterCodeBlock, None),
        (EventType::Separator, None),
        (EventType::Text, Some("```")),
        (EventType::Exit, None)])]
    #[case(vec!["```info\ncode\n```"], vec![
        (EventType::EnterCodeBlock, None),
        (EventType::Text, Some("info")),
        (EventType::Separator, None),
        (EventType::Text, Some("code")),
        (EventType::Exit, None)])]
    #[case(vec!["```\ncode\n\n```"], vec![
        (EventType::EnterCodeBlock, None),
        (EventType::Separator, None),
        (EventType::Text, Some("code")),
        (EventType::LineFeed, None),
        (EventType::Exit, None)])]
    #[case(vec!["```\ncode\n\n\n```"], vec![
        (EventType::EnterCodeBlock, None),
        (EventType::Separator, None),
        (EventType::Text, Some("code")),
        (EventType::LineFeed, None),
        (EventType::LineFeed, None),
        (EventType::Exit, None)])]
    #[case(vec!["```\ncode\nline 3\n```"], vec![
        (EventType::EnterCodeBlock, None),
        (EventType::Separator, None),
        (EventType::Text, Some("code")),
        (EventType::LineFeed, None),
        (EventType::Text, Some("line 3")),
        (EventType::Exit, None)])]
    #[case(vec!["```\ncode\n\nline 3\n```"], vec![
        (EventType::EnterCodeBlock, None),
        (EventType::Separator, None),
        (EventType::Text, Some("code")),
        (EventType::LineFeed, None),
        (EventType::LineFeed, None),
        (EventType::Text, Some("line 3")),
        (EventType::Exit, None)])]
    // ### 代码块与全局阶段语法的互动
    #[case(vec!["```\n<` ``` `>\n```"], vec![
        (EventType::EnterCodeBlock, None),
        (EventType::Separator, None),
        (EventType::Text, Some("```")),
        (EventType::Exit, None)])]
    #[case(vec!["```info<`\ninfo line 2`>\n```"], vec![
        (EventType::EnterCodeBlock, None),
        (EventType::Text, Some("info")),
        (EventType::Text, Some("\ninfo line 2")),
        (EventType::Separator, None),
        (EventType::Exit, None)])]

    fn it_works(#[case] inputs: Vec<&str>, #[case] expected: Vec<EventCase>) {
        for input in inputs {
            let global_parser = global::Parser::new(input.as_bytes(), 0);
            let block_parser = Parser::new(input.as_bytes(), global_parser);

            let actual: Vec<_> = block_parser
                .map(|ev| -> EventCase {
                    (
                        EventType::from(ev.discriminant()),
                        ev.content(input.as_bytes()),
                    )
                })
                .collect();

            assert_eq!(expected, actual)
        }
    }
}
