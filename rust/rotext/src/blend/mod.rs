use std::{cell::RefCell, iter::Peekable, rc::Rc};

use crate::{
    events::{BlendEvent, BlockEvent, Event, InlineLevelParseInputEvent},
    inline::{self},
};

enum State<'a, TBlockParser: Iterator<Item = crate::Result<BlockEvent>>> {
    Normal(Option<Box<Peekable<TBlockParser>>>),
    TakenOver(inline::Parser<'a, WhileInlineSegment<TBlockParser>>),
}

/// 用于将 “产出 [BlockEvent] 的流” 中每段 “留给行内阶段解析器处理的、连续产出
/// 的事件” 截取为单独的流提供给使用者。使用者需要将提供的那些流映射为新的流。
pub struct BlockEventStreamInlineSegmentMapper<
    'a,
    TBlockParser: Iterator<Item = crate::Result<BlockEvent>>,
> {
    input: &'a [u8],
    state: State<'a, TBlockParser>,
    event_stream_returner: Rc<RefCell<Option<Box<Peekable<TBlockParser>>>>>,
}

impl<'a, TBlockParser: Iterator<Item = crate::Result<BlockEvent>>>
    BlockEventStreamInlineSegmentMapper<'a, TBlockParser>
{
    pub fn new(input: &'a [u8], event_stream: TBlockParser) -> Self {
        Self {
            input,
            state: State::Normal(Some(Box::new(event_stream.peekable()))),
            event_stream_returner: Rc::new(RefCell::new(None)),
        }
    }

    #[inline(always)]
    fn next(&mut self) -> Option<crate::Result<BlendEvent>> {
        let ret = loop {
            match self.state {
                State::Normal(ref mut event_stream) => {
                    let event_stream_unchecked =
                        unsafe { event_stream.as_mut().unwrap_unchecked() };
                    let next = match event_stream_unchecked.next() {
                        Some(Ok(x)) => x,
                        Some(Err(err)) => return Some(Err(err)),
                        None => return None,
                    };
                    if next.opens_inline_phase() {
                        let segment_stream = WhileInlineSegment::new(
                            unsafe { event_stream.take().unwrap_unchecked() },
                            self.event_stream_returner.clone(),
                        );
                        let inline_parser = inline::Parser::new(self.input, segment_stream);
                        self.state = State::TakenOver(inline_parser);
                        break Ok(BlendEvent::try_from(Event::from(next)).unwrap());
                    } else {
                        break Ok(BlendEvent::try_from(Event::from(next)).unwrap());
                    }
                }
                State::TakenOver(ref mut inline_parser) => {
                    if let Some(next) = inline_parser.next() {
                        break Ok(BlendEvent::try_from(Event::from(next)).unwrap());
                    } else {
                        let event_stream = self.event_stream_returner.borrow_mut().take().unwrap();
                        self.state = State::Normal(Some(event_stream));
                    }
                }
            };
        };

        Some(ret)
    }
}

impl<'a, TBlockParser: Iterator<Item = crate::Result<BlockEvent>>> Iterator
    for BlockEventStreamInlineSegmentMapper<'a, TBlockParser>
{
    type Item = crate::Result<BlendEvent>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next()
    }
}

pub struct WhileInlineSegment<TBlockParser: Iterator<Item = crate::Result<BlockEvent>>> {
    inner: Option<Box<Peekable<TBlockParser>>>,
    event_stream_returner: Rc<RefCell<Option<Box<Peekable<TBlockParser>>>>>,
}

impl<TBlockParser: Iterator<Item = crate::Result<BlockEvent>>> WhileInlineSegment<TBlockParser> {
    fn new(
        inner: Box<Peekable<TBlockParser>>,
        event_stream_returner: Rc<RefCell<Option<Box<Peekable<TBlockParser>>>>>,
    ) -> Self {
        Self {
            inner: Some(inner),
            event_stream_returner,
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
            *self.event_stream_returner.borrow_mut() = self.inner.take();
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
