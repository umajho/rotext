#[cfg(test)]
mod tests;

use crate::{
    events::{BlendEvent, BlockEvent, Event, InlineInputEvent},
    inline::{self},
    utils::{internal::peekable::Peekable, stack::Stack},
};

#[allow(clippy::large_enum_variant)]
enum State<
    'a,
    TBlockParser: Iterator<Item = crate::Result<BlockEvent>>,
    TInlineStack: Stack<inline::StackEntry>,
> {
    /// Option 仅用于处理所有权，`None` 为无效状态。
    Normal(Option<TBlockParser>),
    ParsingInline {
        inline_parser: inline::Parser<'a, TInlineStack>,
        /// Option 仅用于处理所有权，`None` 为无效状态。
        segment_stream: Option<Peekable<2, WhileInlineSegment<TBlockParser>>>,
    },
}

/// 用于将 “产出 [BlockEvent] 的流” 中每段 “留给行内阶段解析器处理的、连续产出
/// 的事件” 截取为单独的流提供给使用者。使用者需要将提供的那些流映射为新的流。
pub struct BlockEventStreamInlineSegmentMapper<
    'a,
    TBlockParser: Iterator<Item = crate::Result<BlockEvent>>,
    TInlineStack: Stack<inline::StackEntry>,
> {
    input: &'a [u8],
    state: State<'a, TBlockParser, TInlineStack>,
}

impl<
        'a,
        TBlockParser: Iterator<Item = crate::Result<BlockEvent>>,
        TInlineStack: Stack<inline::StackEntry>,
    > BlockEventStreamInlineSegmentMapper<'a, TBlockParser, TInlineStack>
{
    pub fn new(input: &'a [u8], block_parser: TBlockParser) -> Self {
        Self {
            input,
            state: State::Normal(Some(block_parser)),
        }
    }

    #[inline(always)]
    fn next(&mut self) -> Option<crate::Result<BlendEvent>> {
        let ret = loop {
            match self.state {
                State::Normal(ref mut block_parser) => {
                    let next = match unsafe { block_parser.as_mut().unwrap_unchecked() }.next() {
                        Some(Ok(x)) => x,
                        Some(Err(err)) => return Some(Err(err)),
                        None => return None,
                    };
                    if next.opens_inline_phase() {
                        let block_parser = unsafe { block_parser.take().unwrap_unchecked() };
                        let segment_stream = WhileInlineSegment::new(block_parser);
                        let inline_parser = inline::Parser::new(self.input);
                        self.state = State::ParsingInline {
                            inline_parser,
                            segment_stream: Some(Peekable::new(segment_stream)),
                        };
                        break Ok(BlendEvent::try_from(Event::from(next)).unwrap());
                    } else {
                        break Ok(BlendEvent::try_from(Event::from(next)).unwrap());
                    }
                }
                State::ParsingInline {
                    ref mut inline_parser,
                    ref mut segment_stream,
                } => {
                    match inline_parser.next(unsafe { segment_stream.as_mut().unwrap_unchecked() })
                    {
                        Some(Ok(next)) => {
                            break Ok(BlendEvent::try_from(Event::from(next)).unwrap());
                        }
                        Some(Err(err)) => break Err(err),
                        None => {
                            let segment_stream =
                                unsafe { segment_stream.take().unwrap_unchecked() };
                            let (block_parser, leftover, err) = segment_stream.take_inner().drop();
                            if let Some(err) = err {
                                break Err(err);
                            }
                            self.state = State::Normal(Some(block_parser));

                            if let Some(ev) = leftover {
                                break Ok(BlendEvent::try_from(Event::from(ev)).unwrap());
                            }
                        }
                    }
                }
            };
        };

        Some(ret)
    }
}

impl<
        'a,
        TBlockParser: Iterator<Item = crate::Result<BlockEvent>>,
        TInlineStack: Stack<inline::StackEntry>,
    > Iterator for BlockEventStreamInlineSegmentMapper<'a, TBlockParser, TInlineStack>
{
    type Item = crate::Result<BlendEvent>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next()
    }
}

pub struct WhileInlineSegment<TBlockParser: Iterator<Item = crate::Result<BlockEvent>>> {
    block_parser: TBlockParser,

    leftover: Option<BlockEvent>,
    error: Option<crate::Error>,
}

impl<TBlockParser: Iterator<Item = crate::Result<BlockEvent>>> WhileInlineSegment<TBlockParser> {
    fn new(block_parser: TBlockParser) -> Self {
        Self {
            block_parser,
            leftover: None,
            error: None,
        }
    }

    fn drop(self) -> (TBlockParser, Option<BlockEvent>, Option<crate::Error>) {
        (self.block_parser, self.leftover, self.error)
    }

    #[inline(always)]
    fn next(&mut self) -> Option<InlineInputEvent> {
        match self.block_parser.next() {
            Some(Ok(ev)) => {
                if ev.closes_inline_phase() {
                    self.leftover = Some(ev);
                    None
                } else {
                    Some(InlineInputEvent::try_from(Event::from(ev)).unwrap())
                }
            }
            Some(Err(err)) => {
                self.error = Some(err);
                None
            }
            None => None,
        }
    }
}

impl<TBlockParser: Iterator<Item = crate::Result<BlockEvent>>> Iterator
    for WhileInlineSegment<TBlockParser>
{
    type Item = InlineInputEvent;

    fn next(&mut self) -> Option<Self::Item> {
        self.next()
    }
}
