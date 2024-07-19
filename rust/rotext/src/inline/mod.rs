use crate::{
    blend,
    events::{InlineEvent, InlineLevelParseInputEvent},
};

pub struct Parser<'a> {
    input_stream: blend::WhileInlineSegment<'a>,
}

impl<'a> Parser<'a> {
    pub fn new(input_stream: blend::WhileInlineSegment<'a>) -> Self {
        Self { input_stream }
    }

    #[inline(always)]
    fn next(&mut self) -> Option<InlineEvent> {
        let next = self.input_stream.next()?;

        let to_yield = match next {
            InlineLevelParseInputEvent::Unparsed(content) => InlineEvent::Text(content),
            InlineLevelParseInputEvent::LineBreak => InlineEvent::LineBreak,
            InlineLevelParseInputEvent::Text(content) => InlineEvent::Text(content),
        };

        Some(to_yield)
    }
}

impl<'a> Iterator for Parser<'a> {
    type Item = InlineEvent;

    fn next(&mut self) -> Option<Self::Item> {
        self.next()
    }
}
