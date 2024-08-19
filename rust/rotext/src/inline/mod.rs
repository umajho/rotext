mod parser_inner;
mod stack_wrapper;
mod types;

#[cfg(test)]
mod tests;

use std::ops::Range;

use parser_inner::ParserInner;
use stack_wrapper::{TopLeaf, TopLeafCodeSpan};
use types::{Cursor, YieldContext};

use crate::{
    common::m,
    events::{InlineEvent, InlineLevelParseInputEvent},
    types::{Tym, TYM_UNIT},
    utils::internal::{
        string::{count_continuous_character, count_continuous_character_with_maximum},
        utf8::get_byte_length_by_first_char,
    },
};

pub struct Parser<'a, TInput: Iterator<Item = InlineLevelParseInputEvent>> {
    input: &'a [u8],
    event_stream: TInput,

    state: State<'a>,
    inner: ParserInner,
}

enum State<'a> {
    Idle,
    Parsing { input: &'a [u8], cursor: Cursor },
    ExitingUntilStackIsEmptyAndThenEnd,
    Ended,
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
    fn next(&mut self) -> Option<crate::Result<InlineEvent>> {
        loop {
            if let Some(ev) = self.inner.pop_to_be_yielded() {
                break Some(Ok(ev));
            }

            let tym = match &mut self.state {
                State::ExitingUntilStackIsEmptyAndThenEnd => {
                    let (tym, state) =
                        Self::exit_until_stack_is_empty_and_then_end(&mut self.inner);
                    if let Some(state) = state {
                        self.state = state;
                    }
                    tym.into()
                }
                State::Ended => break None,
                State::Idle => {
                    let Some(next) = self.event_stream.next() else {
                        self.state = if self.inner.stack.is_empty() {
                            State::Ended
                        } else {
                            State::ExitingUntilStackIsEmptyAndThenEnd
                        };

                        continue;
                    };

                    let to_yield = match next {
                        InlineLevelParseInputEvent::Unparsed(content) => {
                            let input = &self.input[..content.end];
                            let cursor = Cursor::new(content.start);
                            self.state = State::Parsing { input, cursor };
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
                State::Parsing { input, cursor } => {
                    if cursor.value() < input.len() {
                        Self::parse(input, cursor, &mut self.inner)
                    } else {
                        self.state = State::Idle;
                        TYM_UNIT.into()
                    }
                }
            };
            self.inner.enforce_to_yield_mark(tym);
        }
    }

    fn parse(input: &[u8], cursor: &mut Cursor, inner: &mut ParserInner) -> Tym<2> {
        match inner.stack.pop_top_leaf() {
            None => Self::parse_normal(input, cursor, inner),
            Some(TopLeaf::CodeSpan(top_leaf)) => {
                leaf::code_span::parse_content_and_process(input, cursor, inner, top_leaf)
            }
        }
    }

    fn parse_normal(input: &[u8], cursor: &mut Cursor, inner: &mut ParserInner) -> Tym<2> {
        let start = cursor.value();
        while let Some(char) = input.get(cursor.value()) {
            match char {
                m!('\\') if cursor.value() < input.len() - 1 => {
                    let tym_a = if cursor.value() > start {
                        inner.r#yield(InlineEvent::Text(start..cursor.value()))
                    } else {
                        TYM_UNIT.into()
                    };

                    let target_first_byte = unsafe { *input.get_unchecked(cursor.value() + 1) };
                    let target_utf8_length = get_byte_length_by_first_char(target_first_byte);

                    let tym_b = inner.r#yield(InlineEvent::Text(
                        (cursor.value() + 1)..(cursor.value() + 1 + target_utf8_length),
                    ));
                    cursor.move_forward(1 + target_utf8_length);

                    return tym_a.add(tym_b).into();
                }
                m!('>') if input.get(cursor.value() + 1) == Some(&m!('>')) => {
                    let text_content = start..cursor.value();
                    cursor.move_forward(">>".len());
                    let start = cursor.value();

                    let ref_link_content =
                        leaf::ref_link::advance_until_potential_content_ends(input, cursor);
                    let tym = if let Some(()) = ref_link_content {
                        let tym_a = if !text_content.is_empty() {
                            inner.r#yield(InlineEvent::Text(text_content))
                        } else {
                            TYM_UNIT.into()
                        };

                        let tym_b = inner.r#yield(InlineEvent::RefLink(start..cursor.value()));

                        tym_a.add(tym_b)
                    } else {
                        continue;
                    };
                    return tym.into();
                }
                m!('[') => match input.get(cursor.value() + 1) {
                    None => {
                        cursor.move_forward(1);
                        break;
                    }
                    Some(m!('=')) => {
                        let tym_a = yield_text_if_not_empty(start, cursor, inner);

                        cursor.move_forward("[=".len());
                        let content = leaf::dicexp::advance_until_ends(input, cursor);
                        let tym_b = inner.r#yield(InlineEvent::Dicexp(content));

                        return tym_a.add(tym_b).into();
                    }
                    Some(m!('`')) => {
                        let tym_a = yield_text_if_not_empty(start, cursor, inner);

                        let backticks = "`".len()
                            + count_continuous_character(
                                input,
                                m!('`'),
                                cursor.value() + "[`".len(),
                            );
                        cursor.move_forward("[".len() + backticks);
                        let top_leaf = TopLeafCodeSpan { backticks };
                        let ev = top_leaf.make_enter_event();
                        inner.stack.push_top_leaf(top_leaf.into());
                        let tym_b = inner.r#yield(ev);

                        return tym_a.add(tym_b).into();
                    }
                    Some(_) => {
                        cursor.move_forward(1);
                        continue;
                    }
                },
                _ => cursor.move_forward(1),
            }
        }

        let tym = inner.r#yield(InlineEvent::Text(start..cursor.value()));
        tym.into()
    }

    fn exit_until_stack_is_empty_and_then_end(
        inner: &mut ParserInner,
    ) -> (Tym<1>, Option<State<'a>>) {
        if let Some(top_leaf) = inner.stack.pop_top_leaf() {
            let tym = match top_leaf {
                stack_wrapper::TopLeaf::CodeSpan(top_leaf) => {
                    inner.r#yield(top_leaf.make_exit_event())
                }
            };
            return (tym, None);
        }

        (TYM_UNIT.into(), Some(State::Ended))
    }
}

impl<'a, TInput: Iterator<Item = InlineLevelParseInputEvent>> Iterator for Parser<'a, TInput> {
    type Item = crate::Result<InlineEvent>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next()
    }
}

fn yield_text_if_not_empty(start: usize, cursor: &Cursor, inner: &mut ParserInner) -> Tym<1> {
    if cursor.value() > start {
        inner.r#yield(InlineEvent::Text(start..cursor.value()))
    } else {
        TYM_UNIT.into()
    }
}

mod leaf {
    use super::*;

    pub mod ref_link {
        use super::*;

        /// 推进游标并尝试解析 ref link 的内容。在成功解析为 ref link 内容时返回 `Some(())`，此时
        /// `ctx.cursor()` 是解析内容的末尾。
        pub fn advance_until_potential_content_ends(
            input: &[u8],
            cursor: &mut Cursor,
        ) -> Option<()> {
            let char = input.get(cursor.value())?;
            if !char.is_ascii_alphabetic() {
                return None;
            }
            cursor.move_forward(1);

            loop {
                let char = input.get(cursor.value())?;
                if char.is_ascii_alphabetic() {
                    cursor.move_forward(1);
                    continue;
                } else if char == &b'.' {
                    cursor.move_forward(1);
                    break;
                } else {
                    return None;
                }
            }

            let char = input.get(cursor.value())?;
            if char.is_ascii_alphabetic() {
                cursor.move_forward(1);
                loop {
                    let Some(char) = input.get(cursor.value()) else {
                        return Some(());
                    };
                    if char.is_ascii_alphabetic() {
                        cursor.move_forward(1);
                        continue;
                    } else if char == &b'#' {
                        cursor.move_forward(1);
                        break;
                    } else {
                        return Some(());
                    }
                }

                match input.get(cursor.value()) {
                    Some(char) if char.is_ascii_digit() => {}
                    _ => {
                        cursor.set_value(cursor.value() - 1);
                        return Some(());
                    }
                };
                cursor.move_forward(1);
            } else if char.is_ascii_digit() {
                cursor.move_forward(1);
            } else {
                return None;
            }

            loop {
                let Some(char) = input.get(cursor.value()) else {
                    return Some(());
                };
                if char.is_ascii_digit() {
                    cursor.move_forward(1);
                    continue;
                } else {
                    return Some(());
                }
            }
        }
    }

    pub mod dicexp {
        use super::*;

        /// 推进游标，直到到了数量匹配的 “]” 之前，或者 `input` 到头时。如果是前者，结束时
        /// `ctx.cursor()` 对应于 “]” 的索引，也即还没消耗掉那个 “]”。
        pub fn advance_until_ends(input: &[u8], cursor: &mut Cursor) -> Range<usize> {
            let start = cursor.value();

            let mut depth = 1;

            while let Some(char) = input.get(cursor.value()) {
                match char {
                    m!('[') => depth += 1,
                    m!(']') => {
                        depth -= 1;
                        if depth == 0 {
                            let content = start..cursor.value();
                            cursor.move_forward(1);
                            return content;
                        }
                    }
                    _ => {}
                }
                cursor.move_forward(1)
            }

            start..cursor.value()
        }
    }

    pub mod code_span {
        use super::*;

        pub fn parse_content_and_process(
            input: &[u8],
            cursor: &mut Cursor,
            inner: &mut ParserInner,
            top_leaf: TopLeafCodeSpan,
        ) -> Tym<2> {
            let start = cursor.value();
            while let Some(&char) = input.get(cursor.value()) {
                if char != m!('`') {
                    cursor.move_forward(1);
                    continue;
                }

                match input.get(cursor.value() + top_leaf.backticks) {
                    None => {
                        cursor.set_value(input.len());
                        continue;
                    }
                    Some(&m!(']')) => {}
                    Some(_) => {
                        cursor.move_forward(1);
                        continue;
                    }
                }

                let actual_backticks = "`".len()
                    + count_continuous_character_with_maximum(
                        input,
                        m!('`'),
                        cursor.value() + 1,
                        top_leaf.backticks - 1,
                    );
                if actual_backticks != top_leaf.backticks {
                    cursor.move_forward(actual_backticks + "]".len());
                    continue;
                }

                let tym_a = yield_text_if_not_empty(start, cursor, inner);

                cursor.move_forward(top_leaf.backticks + "]".len());
                let tym_b = inner.r#yield(top_leaf.make_exit_event());

                return tym_a.add(tym_b);
            }

            inner.stack.push_top_leaf(top_leaf.into());
            let tym = inner.r#yield(InlineEvent::Text(start..cursor.value()));
            tym.into()
        }
    }
}
