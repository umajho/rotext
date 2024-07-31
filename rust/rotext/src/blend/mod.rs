use std::{cell::RefCell, iter::Peekable, rc::Rc};

use crate::{
    events::{BlendEvent, BlockEvent, Event, InlineLevelParseInputEvent},
    inline::{self},
};

enum State<TBlockParser: Iterator<Item = crate::Result<BlockEvent>>> {
    Normal(Option<Box<Peekable<TBlockParser>>>),
    TakenOver(inline::Parser<WhileInlineSegment<TBlockParser>>),
}

/// 用于将 “产出 [BlockEvent] 的流” 中每段 “留给行内阶段解析器处理的、连续产出
/// 的事件” 截取为单独的流提供给使用者。使用者需要将提供的那些流映射为新的流。
pub struct BlockEventStreamInlineSegmentMapper<
    TBlockParser: Iterator<Item = crate::Result<BlockEvent>>,
> {
    state: State<TBlockParser>,
    input_stream_returner: Rc<RefCell<Option<Box<Peekable<TBlockParser>>>>>,
}

impl<TBlockParser: Iterator<Item = crate::Result<BlockEvent>>>
    BlockEventStreamInlineSegmentMapper<TBlockParser>
{
    pub fn new(input_stream: TBlockParser) -> Self {
        Self {
            state: State::Normal(Some(Box::new(input_stream.peekable()))),
            input_stream_returner: Rc::new(RefCell::new(None)),
        }
    }

    #[inline(always)]
    fn next(&mut self) -> Option<crate::Result<BlendEvent>> {
        let ret = loop {
            match self.state {
                State::Normal(ref mut input_stream) => {
                    let input_stream_unchecked =
                        unsafe { input_stream.as_mut().unwrap_unchecked() };
                    let next = match input_stream_unchecked.next() {
                        Some(Ok(x)) => x,
                        Some(Err(err)) => return Some(Err(err)),
                        None => return None,
                    };
                    if next.opens_inline_phase() {
                        let segment_stream = WhileInlineSegment::new(
                            unsafe { input_stream.take().unwrap_unchecked() },
                            self.input_stream_returner.clone(),
                        );
                        let inline_parser = inline::Parser::new(segment_stream);
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
                        let input_stream = self.input_stream_returner.borrow_mut().take().unwrap();
                        self.state = State::Normal(Some(input_stream));
                    }
                }
            };
        };

        Some(ret)
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
