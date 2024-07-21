use std::{cell::RefCell, iter::Peekable, rc::Rc};

use crate::{
    block,
    events::{BlendEvent, Event, InlineLevelParseInputEvent},
    inline::{self},
};

enum State<'a> {
    Normal(Box<Peekable<block::Parser<'a>>>),
    TakenOver(inline::Parser<'a>),

    Invalid,
}

/// 用于将 “产出 [BlockEvent] 的流” 中每段 “留给行内阶段解析器处理的、连续产出
/// 的事件” 截取为单独的流提供给使用者。使用者需要将提供的那些流映射为新的流。
pub struct BlockEventStreamInlineSegmentMapper<'a> {
    map_fn: Box<dyn Fn(WhileInlineSegment<'a>) -> inline::Parser + 'a>,

    state: State<'a>,
    input_stream_returner: Rc<RefCell<Option<Box<Peekable<block::Parser<'a>>>>>>,
}

impl<'a> BlockEventStreamInlineSegmentMapper<'a> {
    pub fn new(
        input_stream: block::Parser<'a>,
        map_fn: Box<dyn Fn(WhileInlineSegment<'a>) -> inline::Parser + 'a>,
    ) -> Self {
        Self {
            map_fn,
            state: State::Normal(Box::new(input_stream.peekable())),
            input_stream_returner: Rc::new(RefCell::new(None)),
        }
    }

    #[inline(always)]
    fn next(&mut self) -> Option<BlendEvent> {
        loop {
            let to_break: Option<BlendEvent>;

            (to_break, self.state) = match std::mem::replace(&mut self.state, State::Invalid) {
                State::Normal(mut input_stream) => {
                    let next = input_stream.next()?;
                    if next.opens_inline_phase() {
                        let segment_stream = WhileInlineSegment::new(
                            input_stream,
                            self.input_stream_returner.clone(),
                        );
                        let inline_parser = (self.map_fn)(segment_stream);
                        (
                            Some(BlendEvent::try_from(Event::from(next)).unwrap()),
                            State::TakenOver(inline_parser),
                        )
                    } else {
                        (
                            Some(BlendEvent::try_from(Event::from(next)).unwrap()),
                            State::Normal(input_stream),
                        )
                    }
                }
                State::TakenOver(mut inline_parser) => {
                    if let Some(next) = inline_parser.next() {
                        (
                            Some(BlendEvent::try_from(Event::from(next)).unwrap()),
                            State::TakenOver(inline_parser),
                        )
                    } else {
                        let input_stream = self.input_stream_returner.borrow_mut().take().unwrap();
                        (None, State::Normal(input_stream))
                    }
                }
                State::Invalid => unreachable!(),
            };

            if let Some(to_break) = to_break {
                break Some(to_break);
            }
        }
    }
}

impl<'a> Iterator for BlockEventStreamInlineSegmentMapper<'a> {
    type Item = BlendEvent;

    fn next(&mut self) -> Option<Self::Item> {
        self.next()
    }
}

pub struct WhileInlineSegment<'a> {
    inner: Option<Box<Peekable<block::Parser<'a>>>>,
    input_stream_returner: Rc<RefCell<Option<Box<Peekable<block::Parser<'a>>>>>>,
}

impl<'a> WhileInlineSegment<'a> {
    fn new(
        inner: Box<Peekable<block::Parser<'a>>>,
        input_stream_returner: Rc<RefCell<Option<Box<Peekable<block::Parser<'a>>>>>>,
    ) -> Self {
        Self {
            inner: Some(inner),
            input_stream_returner,
        }
    }

    #[inline(always)]
    fn next(&mut self) -> Option<InlineLevelParseInputEvent> {
        let peeked = self.inner.as_mut().unwrap().peek();
        if peeked.is_some_and(|p| p.closes_inline_phase()) {
            *self.input_stream_returner.borrow_mut() = self.inner.take();
            None
        } else {
            let next = self.inner.as_mut().unwrap().next();
            next.map(|ev| InlineLevelParseInputEvent::try_from(Event::from(ev)).unwrap())
        }
    }
}

impl<'a> Iterator for WhileInlineSegment<'a> {
    type Item = InlineLevelParseInputEvent;

    fn next(&mut self) -> Option<Self::Item> {
        self.next()
    }
}
