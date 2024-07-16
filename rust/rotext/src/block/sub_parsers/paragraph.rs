use crate::{
    block::{context::Context, sub_parsers, Event},
    global,
};

enum State<'a, I: 'a + Iterator<Item = global::Event>> {
    Initial(Box<Context<'a, I>>),
    Content(Box<sub_parsers::content::Parser<'a, I>>),
    ContentDeferred(Box<sub_parsers::content::Parser<'a, I>>, Event),
    Exiting(Box<Context<'a, I>>),
    Exited(Box<Context<'a, I>>),

    Invalid,
    ShouldBeDropped,
}

pub struct Parser<'a, I: 'a + Iterator<Item = global::Event>> {
    state: State<'a, I>,
}

impl<'a, I: 'a + Iterator<Item = global::Event>> Parser<'a, I> {
    pub fn new(ctx: Box<Context<'a, I>>) -> Self {
        Self {
            state: State::Initial(ctx),
        }
    }

    #[inline(always)]
    fn next(&mut self) -> Option<Event> {
        let ret: Option<Event>;

        let state = std::mem::replace(&mut self.state, State::Invalid);
        (ret, self.state) = match state {
            State::Initial(ctx) => {
                let mut content_parser = Box::new(sub_parsers::content::Parser::new(ctx));
                let next = content_parser.next();
                if let Some(ev) = next {
                    (
                        Some(Event::EnterParagraph),
                        State::ContentDeferred(content_parser, ev),
                    )
                } else {
                    (Some(Event::Exit), State::Exiting((*content_parser).drop()))
                }
            }
            State::Content(mut content_parser) => {
                let next = content_parser.next();
                if next.is_some() {
                    (next, State::Content(content_parser))
                } else {
                    (Some(Event::Exit), State::Exiting((*content_parser).drop()))
                }
            }
            State::ContentDeferred(parser, ev) => (Some(ev), State::Content(parser)),
            State::Exiting(ctx) => (None, State::Exited(ctx)),
            // 当解析器作为迭代器被耗尽而返回 `None` 时，解析器进入此状态（
            // [State::Exited]）。此后，不应该再调用 `next` 方法，否则就会执行到
            // 这里。正确的做法是 `take_context` 取回 [Context]，并将解析器
            // drop 掉。
            State::Exited(_) | State::Invalid | State::ShouldBeDropped => unreachable!(),
        };

        ret
    }
}

impl<'a, I: 'a + Iterator<Item = global::Event>> Iterator for Parser<'a, I> {
    type Item = Event;

    fn next(&mut self) -> Option<Self::Item> {
        self.next()
    }
}

impl<'a, I: 'a + Iterator<Item = global::Event>> sub_parsers::SubParser<'a, I> for Parser<'a, I> {
    fn next(&mut self) -> Option<Event> {
        self.next()
    }

    fn take_context(&mut self) -> Box<Context<'a, I>> {
        let state = std::mem::replace(&mut self.state, State::ShouldBeDropped);
        let State::Exited(ctx) = state else {
            unreachable!()
        };
        ctx
    }
}
