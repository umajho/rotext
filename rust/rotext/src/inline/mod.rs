mod parser_inner;
mod types;

use parser_inner::ParserInner;
use types::{CursorContext, YieldContext};

use crate::{
    common::m,
    events::{InlineEvent, InlineLevelParseInputEvent},
    types::{Tym, TYM_UNIT},
    utils::internal::utf8::get_byte_length_by_first_char,
};

pub struct Parser<'a, TInput: Iterator<Item = InlineLevelParseInputEvent>> {
    input: &'a [u8],
    event_stream: TInput,

    state: State<'a>,
}

enum State<'a> {
    Idle,
    Parsing { input: &'a [u8], inner: ParserInner },
}

impl<'a, TInput: Iterator<Item = InlineLevelParseInputEvent>> Parser<'a, TInput> {
    pub fn new(input: &'a [u8], event_stream: TInput) -> Self {
        Self {
            input,
            event_stream,
            state: State::Idle,
        }
    }

    #[inline(always)]
    fn next(&mut self) -> Option<crate::Result<InlineEvent>> {
        loop {
            match &mut self.state {
                State::Idle => {
                    let next = self.event_stream.next()?;

                    let to_yield = match next {
                        InlineLevelParseInputEvent::Unparsed(content) => {
                            let input = &self.input[..content.end];
                            let inner = ParserInner::new(content.start);
                            self.state = State::Parsing { input, inner };
                            continue;
                        }
                        InlineLevelParseInputEvent::VerbatimEscaping(verbatim_escaping) => {
                            InlineEvent::VerbatimEscaping(verbatim_escaping)
                        }
                        InlineLevelParseInputEvent::NewLine(new_line) => {
                            InlineEvent::NewLine(new_line)
                        }
                    };

                    break Some(Ok(to_yield));
                }
                State::Parsing { input, inner } => {
                    if let Some(ev) = inner.pop_to_be_yielded() {
                        break Some(Ok(ev));
                    }

                    if inner.cursor() < input.len() {
                        let tym = Self::parse(input, inner);
                        inner.enforce_to_yield_mark(tym);
                    } else {
                        self.state = State::Idle;
                    }
                }
            }
        }
    }

    fn parse(input: &[u8], inner: &mut ParserInner) -> Tym<2> {
        let start = inner.cursor();
        while inner.cursor() < input.len() {
            // SAFETY: `inner.cursor()` `input.len()`.
            match unsafe { input.get_unchecked(inner.cursor()) } {
                m!('\\') if inner.cursor() < input.len() - 1 => {
                    let tym_a = if inner.cursor() > start {
                        inner.r#yield(InlineEvent::Text(start..inner.cursor()))
                    } else {
                        TYM_UNIT.into()
                    };

                    let target_first_byte = unsafe { *input.get_unchecked(inner.cursor() + 1) };
                    let target_utf8_length = get_byte_length_by_first_char(target_first_byte);

                    let tym_b = inner.r#yield(InlineEvent::Text(
                        (inner.cursor() + 1)..(inner.cursor() + 1 + target_utf8_length),
                    ));
                    inner.move_cursor_forward(1 + target_utf8_length);

                    return tym_a.add(tym_b).into();
                }
                _ => inner.move_cursor_forward(1),
            }
        }

        let tym = inner.r#yield(InlineEvent::Text(start..inner.cursor()));
        tym.into()
    }
}

impl<'a, TInput: Iterator<Item = InlineLevelParseInputEvent>> Iterator for Parser<'a, TInput> {
    type Item = crate::Result<InlineEvent>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next()
    }
}
