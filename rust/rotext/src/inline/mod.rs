use crate::events::{InlineEvent, InlineLevelParseInputEvent};

pub struct Parser<TInput: Iterator<Item = InlineLevelParseInputEvent>> {
    input_stream: TInput,
}

impl<TInput: Iterator<Item = InlineLevelParseInputEvent>> Parser<TInput> {
    pub fn new(input_stream: TInput) -> Self {
        Self { input_stream }
    }

    #[inline(always)]
    fn next(&mut self) -> Option<InlineEvent> {
        let next = self.input_stream.next()?;

        let to_yield = match next {
            InlineLevelParseInputEvent::Unparsed(content) => InlineEvent::Text(content),
            InlineLevelParseInputEvent::VerbatimEscaping(verbatim_escaping) => {
                InlineEvent::VerbatimEscaping(verbatim_escaping)
            }
            InlineLevelParseInputEvent::NewLine(new_line) => InlineEvent::NewLine(new_line),
            InlineLevelParseInputEvent::Text(content) => InlineEvent::Text(content),
        };

        Some(to_yield)
    }
}

impl<TInput: Iterator<Item = InlineLevelParseInputEvent>> Iterator for Parser<TInput> {
    type Item = InlineEvent;

    fn next(&mut self) -> Option<Self::Item> {
        self.next()
    }
}
