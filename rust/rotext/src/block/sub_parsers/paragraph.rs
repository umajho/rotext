use crate::{
    block::{context::Context, sub_parsers, Event},
    global,
};

enum State {
    Initial,
    Content(Box<sub_parsers::content::Parser>),
    ContentDeferred(Box<sub_parsers::content::Parser>, Event),
    Exiting,
    Exited,

    Paused(Box<sub_parsers::content::Parser>),
    /// 此状态仅在实现 [sub_parsers::SubParser::resume_and_exit] 时设置。其他情
    /// 况下会直接进入 [State::Exiting]。
    ToExit,

    Invalid,
}

pub struct Parser {
    state: State,
}

impl Parser {
    pub fn new() -> Self {
        Self {
            state: State::Initial,
        }
    }

    #[inline(always)]
    fn next<'a, I: 'a + Iterator<Item = global::Event>>(
        &mut self,
        ctx: &mut Context<'a, I>,
    ) -> sub_parsers::Result {
        let ret: sub_parsers::Result;

        let state = std::mem::replace(&mut self.state, State::Invalid);
        (ret, self.state) = match state {
            State::Initial => {
                let mut content_parser = Box::new(sub_parsers::content::Parser::new());
                match content_parser.next(ctx) {
                    sub_parsers::Result::ToYield(ev) => (
                        sub_parsers::Result::ToYield(Event::EnterParagraph),
                        State::ContentDeferred(content_parser, ev),
                    ),
                    // 块级阶段的解析器一定是 peek 到了非空白内容才使用本解析器，
                    // 因此，本解析器的初始状态不可能是空白。
                    sub_parsers::Result::ToPauseForNewLine => unreachable!(),
                    sub_parsers::Result::Done => {
                        (sub_parsers::Result::ToYield(Event::Exit), State::Exiting)
                    }
                }
            }
            State::Content(mut content_parser) => {
                let next = content_parser.next(ctx);
                match next {
                    sub_parsers::Result::ToYield(ev) => (
                        sub_parsers::Result::ToYield(ev),
                        State::Content(content_parser),
                    ),
                    sub_parsers::Result::ToPauseForNewLine => (
                        sub_parsers::Result::ToPauseForNewLine,
                        State::Paused(content_parser),
                    ),
                    sub_parsers::Result::Done => {
                        (sub_parsers::Result::ToYield(Event::Exit), State::Exiting)
                    }
                }
            }
            State::ContentDeferred(parser, ev) => {
                (sub_parsers::Result::ToYield(ev), State::Content(parser))
            }
            State::Exiting => (sub_parsers::Result::Done, State::Exited),
            // 当解析器作为迭代器被耗尽而返回 `None` 时，解析器进入状态
            // [State::Exited]。此后，不应该再调用 `next` 方法，否则就会执行到
            // 这里。正确的做法是 `take_context` 取回 [Context]，并将解析器
            // drop 掉。
            State::Exited | State::Paused(_) | State::Invalid => unreachable!(),
            State::ToExit => (sub_parsers::Result::ToYield(Event::Exit), State::Exiting),
        };

        ret
    }
}

impl<'a, I: 'a + Iterator<Item = global::Event>> sub_parsers::SubParser<'a, I> for Parser {
    fn next(&mut self, ctx: &mut Context<'a, I>) -> sub_parsers::Result {
        self.next(ctx)
    }

    fn resume_from_pause_for_new_line_and_continue(&mut self) {
        let state = std::mem::replace(&mut self.state, State::Invalid);
        let State::Paused(mut content_parser) = state else {
            unreachable!()
        };
        content_parser.resume_from_pause_for_new_line_and_continue();
        self.state = State::Content(content_parser);
    }

    fn resume_from_pause_for_new_line_and_exit(&mut self) {
        if matches!(self.state, State::Paused(_)) {
            self.state = State::ToExit;
        } else {
            unreachable!()
        }
    }
}
