use crate::block::{context::Context, sub_parsers, Event};

enum State {
    /// 构造解析器后，解析器所处的初始状态。此时，其所解析语法的开启部分应已经被
    /// 消耗。
    Initial,
    Content(Box<sub_parsers::content::Parser>),
    Exiting,
    Exited,

    Invalid,
}

pub struct Parser {
    leading_signs: usize,

    state: State,
}

impl Parser {
    pub fn new(leading_signs: usize) -> Self {
        Self {
            leading_signs,

            state: State::Initial,
        }
    }

    #[inline(always)]
    fn next(&mut self, ctx: &mut Context) -> sub_parsers::Result {
        let ret: sub_parsers::Result;

        let state = std::mem::replace(&mut self.state, State::Invalid);
        (ret, self.state) = match state {
            State::Initial => {
                let opts = sub_parsers::content::Options {
                    mode: sub_parsers::content::Mode::Inline,
                    end_conditions: sub_parsers::content::EndConditions {
                        before_new_line: true,
                        after_repetitive_characters: Some(
                            sub_parsers::content::RepetitiveCharactersCondition {
                                at_line_beginning: false,
                                at_line_end_and_with_space_before: true,
                                character: b'=',
                                minimal_count: self.leading_signs,
                            },
                        ),
                        ..Default::default()
                    },
                    ..Default::default()
                };
                let content_parser = sub_parsers::content::Parser::new(opts);
                (
                    sub_parsers::Result::ToYield(match self.leading_signs {
                        1 => Event::EnterHeading1,
                        2 => Event::EnterHeading2,
                        3 => Event::EnterHeading3,
                        4 => Event::EnterHeading4,
                        5 => Event::EnterHeading5,
                        6 => Event::EnterHeading6,
                        _ => unreachable!(),
                    }),
                    State::Content(Box::new(content_parser)),
                )
            }
            State::Content(mut content_parser) => {
                let next = content_parser.next(ctx);
                match next {
                    sub_parsers::Result::ToYield(ev) => (
                        sub_parsers::Result::ToYield(ev),
                        State::Content(content_parser),
                    ),
                    sub_parsers::Result::ToPauseForNewLine => unreachable!(),
                    sub_parsers::Result::Done => {
                        (sub_parsers::Result::ToYield(Event::Exit), State::Exiting)
                    }
                }
            }
            State::Exiting => (sub_parsers::Result::Done, State::Exited),
            // 当解析器作为迭代器被耗尽而返回 `None` 时，解析器进入状态
            // [State::Exited]。此后，不应该再调用 `next` 方法，否则就会执行到
            // 这里。正确的做法是 `take_context` 取回 [Context]，并将解析器
            // drop 掉。
            State::Exited | State::Invalid => unreachable!(),
        };

        ret
    }
}

impl<'a> sub_parsers::SubParser<'a> for Parser {
    fn next(&mut self, ctx: &mut Context<'a>) -> sub_parsers::Result {
        self.next(ctx)
    }

    fn resume_from_pause_for_new_line_and_continue(&mut self) {
        unreachable!()
    }

    fn resume_from_pause_for_new_line_and_exit(&mut self) {
        unreachable!()
    }
}
