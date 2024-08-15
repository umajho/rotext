mod parser_inner;

use parser_inner::ParserInner;

use crate::{
    events::{InlineEvent, InlineLevelParseInputEvent},
    utils::internal::utf8::get_byte_length_by_first_char,
};

pub struct Parser<'a, TInput: Iterator<Item = InlineLevelParseInputEvent>> {
    input: &'a [u8],
    event_stream: TInput,

    state: State,
    inner: ParserInner,
}

enum State {
    Idle,
    Parsing { end: usize, cursor: usize },
}

impl<'a, TInput: Iterator<Item = InlineLevelParseInputEvent>> Parser<'a, TInput> {
    pub fn new(input: &'a [u8], event_stream: TInput) -> Self {
        Self {
            input,
            event_stream,
            state: State::Idle,
            inner: ParserInner::new(),
        }
    }

    #[inline(always)]
    fn next(&mut self) -> Option<InlineEvent> {
        loop {
            if let Some(ev) = self.inner.pop_to_be_yielded() {
                break Some(ev);
            }

            match &mut self.state {
                State::Idle => {
                    let next = self.event_stream.next()?;

                    let to_yield = match next {
                        InlineLevelParseInputEvent::Unparsed(content) => {
                            let end = content.end;
                            let cursor = content.start;
                            self.state = State::Parsing { end, cursor };
                            continue;
                        }
                        InlineLevelParseInputEvent::VerbatimEscaping(verbatim_escaping) => {
                            InlineEvent::VerbatimEscaping(verbatim_escaping)
                        }
                        InlineLevelParseInputEvent::NewLine(new_line) => {
                            InlineEvent::NewLine(new_line)
                        }
                    };

                    break Some(to_yield);
                }
                State::Parsing { end, cursor } => {
                    if cursor < end {
                        Self::parse(self.input, &mut self.inner, *end, cursor)
                    } else {
                        self.state = State::Idle;
                    }
                }
            }
        }
    }

    fn parse(input: &[u8], inner: &mut ParserInner, end: usize, cursor: &mut usize) {
        let start = *cursor;
        while *cursor < end {
            // SAFETY: `*cursor` < `end` < `input.len()`.
            match unsafe { input.get_unchecked(*cursor) } {
                b'\\' if *cursor < end - 1 => {
                    if *cursor > start {
                        inner.r#yield(InlineEvent::Text(start..*cursor));
                    }

                    let target_first_byte = unsafe { *input.get_unchecked(*cursor + 1) };
                    let target_utf8_length = get_byte_length_by_first_char(target_first_byte);

                    inner.r#yield(InlineEvent::Text(
                        (*cursor + 1)..(*cursor + 1 + target_utf8_length),
                    ));
                    *cursor += 1 + target_utf8_length;
                    return;
                }
                _ => *cursor += 1,
            }
        }
        inner.r#yield(InlineEvent::Text(start..*cursor));
    }
}

impl<'a, TInput: Iterator<Item = InlineLevelParseInputEvent>> Iterator for Parser<'a, TInput> {
    type Item = InlineEvent;

    fn next(&mut self) -> Option<Self::Item> {
        self.next()
    }
}
