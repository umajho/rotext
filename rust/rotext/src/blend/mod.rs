#[cfg(test)]
mod tests;

#[cfg(debug_assertions)]
use crate::events::is_event_of;
use crate::{
    events::{ev, Event},
    inline::{self},
    utils::{internal::peekable::Peekable, stack::Stack},
};

/// `TBlockParser` 迭代的事件应该属于 `Block` 分组。
#[allow(clippy::large_enum_variant)]
enum State<
    'a,
    TBlockParser: Iterator<Item = crate::Result<Event>>,
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
    TBlockParser: Iterator<Item = crate::Result<Event>>,
    TInlineStack: Stack<inline::StackEntry>,
> {
    input: &'a [u8],
    state: State<'a, TBlockParser, TInlineStack>,
}

impl<
        'a,
        TBlockParser: Iterator<Item = crate::Result<Event>>,
        TInlineStack: Stack<inline::StackEntry>,
    > BlockEventStreamInlineSegmentMapper<'a, TBlockParser, TInlineStack>
{
    pub fn new(input: &'a [u8], block_parser: TBlockParser) -> Self {
        Self {
            input,
            state: State::Normal(Some(block_parser)),
        }
    }

    /// 返回的事件属于 `Blend` 分组。
    #[inline(always)]
    fn next(&mut self) -> Option<crate::Result<Event>> {
        let ret = loop {
            match self.state {
                State::Normal(ref mut block_parser) => {
                    let next = match unsafe { block_parser.as_mut().unwrap_unchecked() }.next() {
                        Some(Ok(x)) => x,
                        Some(Err(err)) => return Some(Err(err)),
                        None => return None,
                    };
                    if next.is_block_event_that_opens_inline_phase() {
                        let block_parser = unsafe { block_parser.take().unwrap_unchecked() };
                        let segment_stream = WhileInlineSegment::new(block_parser);
                        let inline_parser = inline::Parser::new(self.input);
                        self.state = State::ParsingInline {
                            inline_parser,
                            segment_stream: Some(Peekable::new(segment_stream)),
                        };
                    }

                    debug_assert!(!matches!(next, ev!(Block, __Unparsed(_))));
                    #[cfg(debug_assertions)]
                    debug_assert!(is_event_of!(Blend, next));
                    // 排除掉 `__Unparsed`（不会在本状态中遇到），属于 `Block` 分
                    // 组的事件都属于 `Blend` 分组，因此分组正确。
                    break Ok(next);
                }
                State::ParsingInline {
                    ref mut inline_parser,
                    ref mut segment_stream,
                } => {
                    match inline_parser.next(unsafe { segment_stream.as_mut().unwrap_unchecked() })
                    {
                        Some(Ok(next)) => {
                            #[cfg(debug_assertions)]
                            debug_assert!(is_event_of!(Blend, next));
                            // 属于 `Inline` 分组的事件都属于 `Blend` 分组，因此
                            // 分组正确。
                            break Ok(next);
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
                                #[cfg(debug_assertions)]
                                debug_assert!(is_event_of!(Blend, ev));
                                // `leftover` 也能保证属于 `Blend` 分组，因此分
                                // 组正确。
                                break Ok(ev);
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
        // 承载的事件属于 `Block` 分组。
        TBlockParser: Iterator<Item = crate::Result<Event>>,
        TInlineStack: Stack<inline::StackEntry>,
    > Iterator for BlockEventStreamInlineSegmentMapper<'_, TBlockParser, TInlineStack>
{
    /// 承载的事件属于 `Blend` 分组。
    type Item = crate::Result<Event>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next()
    }
}

pub struct WhileInlineSegment<
    // 承载的事件属于 `Block` 分组。
    TBlockParser: Iterator<Item = crate::Result<Event>>,
> {
    block_parser: TBlockParser,

    /// 承载的事件属于 `Block` 分组，且不会是 `Unparsed`。即承载的事件也属于
    /// `Blend` 分组。
    leftover: Option<Event>,
    error: Option<crate::Error>,
}

impl<
        // 承载的事件属于 `Block` 分组。
        TBlockParser: Iterator<Item = crate::Result<Event>>,
    > WhileInlineSegment<TBlockParser>
{
    fn new(block_parser: TBlockParser) -> Self {
        Self {
            block_parser,
            leftover: None,
            error: None,
        }
    }

    /// 作为元组第二个元素返回的事件属于 `Block` 分组。
    fn drop(self) -> (TBlockParser, Option<Event>, Option<crate::Error>) {
        (self.block_parser, self.leftover, self.error)
    }

    /// 返回的事件属于 `InlineInput` 分组。
    #[inline(always)]
    fn next(&mut self) -> Option<Event> {
        match self.block_parser.next() {
            Some(Ok(ev)) => {
                if ev.is_block_event_that_closes_inline_phase() {
                    self.leftover = Some(ev);
                    None
                } else {
                    #[cfg(debug_assertions)]
                    debug_assert!(is_event_of!(InlineInput, ev));
                    Some(ev)
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

impl<
        // 承载的事件属于 `Block` 分组。
        TBlockParser: Iterator<Item = crate::Result<Event>>,
    > Iterator for WhileInlineSegment<TBlockParser>
{
    /// 属于 `InlineInput` 分组。
    type Item = Event;

    fn next(&mut self) -> Option<Self::Item> {
        self.next()
    }
}
