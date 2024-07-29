use std::{cell::RefCell, iter::Peekable, rc::Rc};

use crate::{
    events::{BlendEvent, BlockEvent, Event, InlineLevelParseInputEvent},
    inline::{self},
};

enum State<TBlockParser: Iterator<Item = crate::Result<BlockEvent>>> {
    Normal(Box<Peekable<TBlockParser>>),
    TakenOver(inline::Parser<WhileInlineSegment<TBlockParser>>),

    Invalid,
}

/// 用于将 “产出 [BlockEvent] 的流” 中每段 “留给行内阶段解析器处理的、连续产出
/// 的事件” 截取为单独的流提供给使用者。使用者需要将提供的那些流映射为新的流。
pub struct BlockEventStreamInlineSegmentMapper<
    TBlockParser: Iterator<Item = crate::Result<BlockEvent>>,
> {
    state: State<TBlockParser>,
    input_stream_returner: Rc<RefCell<Option<Box<Peekable<TBlockParser>>>>>,
}

impl<'a, TBlockParser: Iterator<Item = crate::Result<BlockEvent>>>
    BlockEventStreamInlineSegmentMapper<TBlockParser>
{
    pub fn new(input_stream: TBlockParser) -> Self {
        Self {
            state: State::Normal(Box::new(input_stream.peekable())),
            input_stream_returner: Rc::new(RefCell::new(None)),
        }
    }

    #[inline(always)]
    fn next(&mut self) -> Option<crate::Result<BlendEvent>> {
        let ret = loop {
            let to_break: Option<BlendEvent>;

            (to_break, self.state) = match std::mem::replace(&mut self.state, State::Invalid) {
                State::Normal(mut input_stream) => {
                    let next = match input_stream.next() {
                        Some(Ok(x)) => x,
                        Some(Err(err)) => return Some(Err(err)),
                        None => break None,
                    };
                    if next.opens_inline_phase() {
                        let segment_stream = WhileInlineSegment::new(
                            input_stream,
                            self.input_stream_returner.clone(),
                        );
                        let inline_parser = inline::Parser::new(segment_stream);
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
        };

        match ret {
            Some(x) => Some(Ok(x)),
            None => None,
        }
    }
}

impl<TBlockParser: Iterator<Item = crate::Result<BlockEvent>>> Iterator
    for BlockEventStreamInlineSegmentMapper<TBlockParser>
{
    type Item = crate::Result<BlendEvent>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next()
    }
}

pub struct WhileInlineSegment<TBlockParser: Iterator<Item = crate::Result<BlockEvent>>> {
    inner: Option<Box<Peekable<TBlockParser>>>,
    input_stream_returner: Rc<RefCell<Option<Box<Peekable<TBlockParser>>>>>,
}

impl<TBlockParser: Iterator<Item = crate::Result<BlockEvent>>> WhileInlineSegment<TBlockParser> {
    fn new(
        inner: Box<Peekable<TBlockParser>>,
        input_stream_returner: Rc<RefCell<Option<Box<Peekable<TBlockParser>>>>>,
    ) -> Self {
        Self {
            inner: Some(inner),
            input_stream_returner,
        }
    }

    #[inline(always)]
    fn next(&mut self) -> Option<InlineLevelParseInputEvent> {
        let peeked = self.inner.as_mut().unwrap().peek();
        let Some(Ok(peeked)) = peeked else {
            self.inner.as_mut().unwrap().next();
            return None;
        };
        if peeked.closes_inline_phase() {
            *self.input_stream_returner.borrow_mut() = self.inner.take();
            None
        } else {
            let next = self.inner.as_mut().unwrap().next().unwrap().unwrap();
            Some(InlineLevelParseInputEvent::try_from(Event::from(next)).unwrap())
        }
    }
}

impl<TBlockParser: Iterator<Item = crate::Result<BlockEvent>>> Iterator
    for WhileInlineSegment<TBlockParser>
{
    type Item = InlineLevelParseInputEvent;

    fn next(&mut self) -> Option<Self::Item> {
        self.next()
    }
}
