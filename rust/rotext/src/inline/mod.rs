mod parser_inner;
mod types;

#[cfg(test)]
mod tests;

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
            // SAFETY: `inner.cursor()` < `input.len()`.
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
                m!('>') if input.get(inner.cursor() + 1) == Some(&m!('>')) => {
                    let text_content = start..inner.cursor();
                    inner.move_cursor_forward(">>".len());
                    let start = inner.cursor();

                    let ref_link_content =
                        advance_until_potential_ref_link_content_ends(input, inner);
                    let tym = if let Some(()) = ref_link_content {
                        let tym_a = if !text_content.is_empty() {
                            inner.r#yield(InlineEvent::Text(text_content))
                        } else {
                            TYM_UNIT.into()
                        };

                        let tym_b = inner.r#yield(InlineEvent::RefLink(start..inner.cursor()));

                        tym_a.add(tym_b)
                    } else {
                        continue;
                    };
                    return tym.into();
                }
                m!('[') => match input.get(inner.cursor() + 1) {
                    None => {
                        inner.move_cursor_forward(1);
                        break;
                    }
                    Some(m!('=')) => {
                        let tym_a = if inner.cursor() > start {
                            inner.r#yield(InlineEvent::Text(start..inner.cursor()))
                        } else {
                            TYM_UNIT.into()
                        };

                        inner.move_cursor_forward("[=".len());
                        let start = inner.cursor();
                        advance_until_dicexp_will_be_ended(input, inner);
                        let tym_b = inner.r#yield(InlineEvent::Dicexp(start..inner.cursor()));
                        inner.move_cursor_forward("]".len());

                        return tym_a.add(tym_b).into();
                    }
                    Some(_) => {
                        inner.move_cursor_forward(1);
                        continue;
                    }
                },
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

/// 推进游标并尝试解析 ref link 的内容。在成功解析为 ref link 内容时返回 `Some(())`，此时
/// `ctx.cursor()` 是解析内容的末尾。
fn advance_until_potential_ref_link_content_ends<TCtx: CursorContext>(
    input: &[u8],
    ctx: &mut TCtx,
) -> Option<()> {
    let char = input.get(ctx.cursor())?;
    if !char.is_ascii_alphabetic() {
        return None;
    }
    ctx.move_cursor_forward(1);

    loop {
        let char = input.get(ctx.cursor())?;
        if char.is_ascii_alphabetic() {
            ctx.move_cursor_forward(1);
            continue;
        } else if char == &b'.' {
            ctx.move_cursor_forward(1);
            break;
        } else {
            return None;
        }
    }

    let char = input.get(ctx.cursor())?;
    if char.is_ascii_alphabetic() {
        ctx.move_cursor_forward(1);
        loop {
            let Some(char) = input.get(ctx.cursor()) else {
                return Some(());
            };
            if char.is_ascii_alphabetic() {
                ctx.move_cursor_forward(1);
                continue;
            } else if char == &b'#' {
                ctx.move_cursor_forward(1);
                break;
            } else {
                return Some(());
            }
        }

        match input.get(ctx.cursor()) {
            Some(char) if char.is_ascii_digit() => {}
            _ => {
                ctx.set_cursor(ctx.cursor() - 1);
                return Some(());
            }
        };
        ctx.move_cursor_forward(1);
    } else if char.is_ascii_digit() {
        ctx.move_cursor_forward(1);
    } else {
        return None;
    }

    loop {
        let Some(char) = input.get(ctx.cursor()) else {
            return Some(());
        };
        if char.is_ascii_digit() {
            ctx.move_cursor_forward(1);
            continue;
        } else {
            return Some(());
        }
    }
}

/// 推进游标，直到到了数量匹配的 “]” 之前，或者 `input` 到头时。如果是前者，结束时
/// `ctx.cursor()` 对应于 “]” 的索引，也即还没消耗掉那个 “]”。
fn advance_until_dicexp_will_be_ended<TCtx: CursorContext>(input: &[u8], ctx: &mut TCtx) {
    let mut depth = 1;

    while ctx.cursor() < input.len() {
        // SAFETY: `inner.cursor()` < `input.len()`.
        match unsafe { input.get_unchecked(ctx.cursor()) } {
            m!('[') => depth += 1,
            m!(']') => {
                depth -= 1;
                if depth == 0 {
                    return;
                }
            }
            _ => {}
        }
        ctx.move_cursor_forward(1)
    }
}
