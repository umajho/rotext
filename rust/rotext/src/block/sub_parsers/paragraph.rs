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
    ) -> Option<Event> {
        let ret: Option<Event>;

        let state = std::mem::replace(&mut self.state, State::Invalid);
        (ret, self.state) = match state {
            State::Initial => {
                let mut content_parser = Box::new(sub_parsers::content::Parser::new());
                let next = content_parser.next(ctx);
                if let Some(ev) = next {
                    (
                        Some(Event::EnterParagraph),
                        State::ContentDeferred(content_parser, ev),
                    )
                } else {
                    (Some(Event::Exit), State::Exiting)
                }
            }
            State::Content(mut content_parser) => {
                let next = content_parser.next(ctx);
                if next.is_some() {
                    (next, State::Content(content_parser))
                } else {
                    (Some(Event::Exit), State::Exiting)
                }
            }
            State::ContentDeferred(parser, ev) => (Some(ev), State::Content(parser)),
            State::Exiting => (None, State::Exited),
            // 当解析器作为迭代器被耗尽而返回 `None` 时，解析器进入此状态（
            // [State::Exited]）。此后，不应该再调用 `next` 方法，否则就会执行到
            // 这里。正确的做法是 `take_context` 取回 [Context]，并将解析器
            // drop 掉。
            State::Exited | State::Invalid => unreachable!(),
        };

        ret
    }
}

impl<'a, I: 'a + Iterator<Item = global::Event>> sub_parsers::SubParser<'a, I> for Parser {
    fn next(&mut self, ctx: &mut Context<'a, I>) -> Option<Event> {
        self.next(ctx)
    }
}
