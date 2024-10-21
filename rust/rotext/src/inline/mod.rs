mod parser_inner;
mod stack_wrapper;
mod types;

#[cfg(test)]
mod tests;

pub use stack_wrapper::StackEntry;

use std::ops::Range;

use crate::events::NewLine;
use parser_inner::ParserInner;
use stack_wrapper::{TopLeaf, TopLeafCodeSpan};
use types::{Cursor, YieldContext};

use crate::{
    blend::Peekable,
    common::m,
    events::{InlineEvent, InlinePhaseParseInputEvent},
    types::{Tym, TYM_UNIT},
    utils::{
        internal::{
            string::{count_continuous_character, count_continuous_character_with_maximum},
            utf8::get_byte_length_by_first_char,
        },
        stack::Stack,
    },
};

pub struct Parser<'a, TInlineStack: Stack<StackEntry>> {
    input: &'a [u8],

    state: State<'a>,
    inner: ParserInner<TInlineStack>,
}

enum State<'a> {
    Idle,
    Parsing { input: &'a [u8], cursor: Cursor },
    ExitingUntilStackIsEmptyAndThenEnd,
    Ended,
}

impl<'a, TInlineStack: Stack<StackEntry>> Parser<'a, TInlineStack> {
    pub fn new(input: &'a [u8]) -> Self {
        Self {
            input,
            state: State::Idle,
            inner: ParserInner::new(),
        }
    }

    pub fn next(
        &mut self,
        event_stream: &mut (impl Iterator<Item = InlinePhaseParseInputEvent>
                  + Peekable<InlinePhaseParseInputEvent>),
    ) -> Option<crate::Result<InlineEvent>> {
        loop {
            if let Some(ev) = self.inner.pop_to_be_yielded() {
                break Some(Ok(ev));
            }

            let result = match &mut self.state {
                State::ExitingUntilStackIsEmptyAndThenEnd => {
                    let (tym, state) =
                        Self::exit_until_stack_is_empty_and_then_end(&mut self.inner);
                    if let Some(state) = state {
                        self.state = state;
                    }
                    Ok(tym.into())
                }
                State::Ended => break None,
                State::Idle => {
                    let Some(next) = event_stream.next() else {
                        self.state = if self.inner.stack.is_empty() {
                            State::Ended
                        } else {
                            State::ExitingUntilStackIsEmptyAndThenEnd
                        };

                        continue;
                    };

                    if self.inner.pop_should_skip_next_input_event() {
                        continue;
                    }

                    let to_yield = match next {
                        InlinePhaseParseInputEvent::Unparsed(content) => {
                            let input = &self.input[..content.end];
                            let cursor = Cursor::new(content.start);
                            self.state = State::Parsing { input, cursor };
                            continue;
                        }
                        InlinePhaseParseInputEvent::VerbatimEscaping(verbatim_escaping) => {
                            InlineEvent::VerbatimEscaping(verbatim_escaping)
                        }
                        InlinePhaseParseInputEvent::NewLine(new_line) => {
                            InlineEvent::NewLine(new_line)
                        }
                    };

                    break Some(Ok(to_yield));
                }
                State::Parsing { input, cursor } => {
                    if cursor.value() < input.len() {
                        Self::parse(input, cursor, &mut self.inner, event_stream)
                    } else {
                        self.state = State::Idle;
                        Ok(TYM_UNIT.into())
                    }
                }
            };
            match result {
                Ok(tym) => self.inner.enforce_to_yield_mark(tym),
                Err(err) => break Some(Err(err)),
            }
        }
    }

    fn parse(
        input: &[u8],
        cursor: &mut Cursor,
        inner: &mut ParserInner<TInlineStack>,
        event_stream: &mut (impl Iterator<Item = InlinePhaseParseInputEvent>
                  + Peekable<InlinePhaseParseInputEvent>),
    ) -> crate::Result<Tym<2>> {
        match inner.stack.pop_top_leaf() {
            None => Self::parse_normal(input, cursor, inner, event_stream),
            Some(TopLeaf::CodeSpan(top_leaf)) => {
                leaf::code_span::parse_content_and_process(input, cursor, inner, top_leaf)
            }
        }
    }

    fn parse_normal(
        input: &[u8],
        cursor: &mut Cursor,
        inner: &mut ParserInner<TInlineStack>,
        event_stream: &mut (impl Iterator<Item = InlinePhaseParseInputEvent>
                  + Peekable<InlinePhaseParseInputEvent>),
    ) -> crate::Result<Tym<2>> {
        let end_condition = inner.stack.make_end_condition();

        let text_start = cursor.value();
        let (text_end, to_yield_after_text): (usize, Option<InlineEvent>) = loop {
            let Some(char) = input.get(cursor.value()) else {
                break (cursor.value(), None);
            };

            match char {
                m!('\\') if cursor.value() < input.len() - 1 => {
                    break special::process_backslash_escaping(input, cursor);
                }
                m!('\\') => match event_stream.peek() {
                    Some(InlinePhaseParseInputEvent::NewLine(new_line)) => {
                        let new_line = new_line.clone();
                        break special::process_hard_break_mark(input, cursor, inner, new_line);
                    }
                    _ => {
                        cursor.move_forward(1);
                        continue;
                    }
                },
                m!('_') if cursor.value() == input.len() - 1 => match event_stream.peek() {
                    Some(InlinePhaseParseInputEvent::NewLine(_)) => {
                        break special::process_lines_joint_mark(input, cursor, inner);
                    }
                    _ => {
                        cursor.move_forward(1);
                        continue;
                    }
                },
                m!('&') if input.get(cursor.value() + 1) == Some(&m!('#')) => {
                    match special::process_potential_numeric_character_reference(input, cursor) {
                        Some(result) => {
                            break result;
                        }
                        None => continue,
                    }
                }
                m!('>') if input.get(cursor.value() + 1) == Some(&m!('>')) => {
                    match leaf::ref_link::process_potential(input, cursor) {
                        Some(result) => {
                            break result;
                        }
                        None => continue,
                    }
                }
                m!('[') => match input.get(cursor.value() + 1) {
                    None => {
                        cursor.move_forward(1);
                        break (cursor.value(), None);
                    }
                    Some(m!('=')) => {
                        break leaf::dicexp::process(input, cursor);
                    }
                    Some(m!('`')) => {
                        break leaf::code_span::process(input, cursor, inner);
                    }
                    Some(m!('\'')) => {
                        let text_end = cursor.value();

                        cursor.move_forward("['".len());
                        inner.stack.push_entry(StackEntry::Strong)?;
                        let to_yield_after_text = InlineEvent::EnterStrong;

                        break (text_end, Some(to_yield_after_text));
                    }
                    Some(m!('~')) => {
                        let text_end = cursor.value();

                        cursor.move_forward("[~".len());
                        inner.stack.push_entry(StackEntry::Strikethrough)?;
                        let to_yield_after_text = InlineEvent::EnterStrikethrough;

                        break (text_end, Some(to_yield_after_text));
                    }
                    Some(_) => {
                        cursor.move_forward(1);
                        continue;
                    }
                },
                &char => {
                    if ((end_condition.on_strong_closing && char == m!('\''))
                        || (end_condition.on_strikethrough_closing && char == m!('~')))
                        && input.get(cursor.value() + 1) == Some(&m!(']'))
                    {
                        let text_end = cursor.value();

                        cursor.move_forward(2);
                        inner.stack.pop_entry();
                        let to_yield_after_text = InlineEvent::ExitInline;

                        break (text_end, Some(to_yield_after_text));
                    }

                    cursor.move_forward(1);
                }
            }
        };

        let tym_a = yield_text_if_not_empty(text_start, text_end, inner);

        let tym_b = if let Some(ev) = to_yield_after_text {
            inner.r#yield(ev)
        } else {
            TYM_UNIT.into()
        };

        Ok(tym_a.add(tym_b))
    }

    fn exit_until_stack_is_empty_and_then_end(
        inner: &mut ParserInner<TInlineStack>,
    ) -> (Tym<1>, Option<State<'a>>) {
        if let Some(top_leaf) = inner.stack.pop_top_leaf() {
            let tym = match top_leaf {
                stack_wrapper::TopLeaf::CodeSpan(top_leaf) => {
                    inner.r#yield(top_leaf.make_exit_event())
                }
            };
            (tym, None)
        } else if let Some(entry) = inner.stack.pop_entry() {
            let tym = match entry {
                StackEntry::Strong | StackEntry::Strikethrough => {
                    inner.r#yield(InlineEvent::ExitInline)
                }
            };

            (tym, None)
        } else {
            (TYM_UNIT.into(), Some(State::Ended))
        }
    }
}

fn yield_text_if_not_empty<TInlineStack: Stack<StackEntry>>(
    start: usize,
    end: usize,
    inner: &mut ParserInner<TInlineStack>,
) -> Tym<1> {
    if end > start {
        inner.r#yield(InlineEvent::Text(start..end))
    } else {
        TYM_UNIT.into()
    }
}

mod special {
    use super::*;

    pub fn process_backslash_escaping(
        input: &[u8],
        cursor: &mut Cursor,
    ) -> (usize, Option<InlineEvent>) {
        let text_end = cursor.value();

        let target_first_byte = unsafe { *input.get_unchecked(cursor.value() + 1) };
        let target_utf8_length = get_byte_length_by_first_char(target_first_byte);

        let to_yield_after_text =
            InlineEvent::Text((cursor.value() + 1)..(cursor.value() + 1 + target_utf8_length));
        cursor.move_forward(1 + target_utf8_length);

        (text_end, Some(to_yield_after_text))
    }

    pub fn process_hard_break_mark<TInlineStack: Stack<StackEntry>>(
        _input: &[u8],
        cursor: &mut Cursor,
        inner: &mut ParserInner<TInlineStack>,
        new_line: NewLine,
    ) -> (usize, Option<InlineEvent>) {
        let text_end = cursor.value();
        cursor.move_forward(1);
        inner.set_should_skip_next_input_event();
        let to_yield_after_text = InlineEvent::NewLine(new_line);

        (text_end, Some(to_yield_after_text))
    }

    pub fn process_lines_joint_mark<TInlineStack: Stack<StackEntry>>(
        _input: &[u8],
        cursor: &mut Cursor,
        inner: &mut ParserInner<TInlineStack>,
    ) -> (usize, Option<InlineEvent>) {
        let text_end = cursor.value();
        cursor.move_forward(1);
        inner.set_should_skip_next_input_event();

        (text_end, None)
    }

    pub fn process_potential_numeric_character_reference(
        input: &[u8],
        cursor: &mut Cursor,
    ) -> Option<(usize, Option<InlineEvent>)> {
        let start = cursor.value();
        cursor.move_forward(2);

        let is_hex = match input.get(cursor.value()) {
            Some(b'x' | b'X') => {
                cursor.move_forward(1);

                let next = input.get(cursor.value())?;
                if !next.is_ascii_hexdigit() {
                    return None;
                }
                cursor.move_forward(1);

                true
            }
            Some(x) if x.is_ascii_digit() => {
                cursor.move_forward(1);
                false
            }
            _ => return None,
        };

        while let Some(char) = input.get(cursor.value()) {
            if *char == m!(';') {
                cursor.move_forward(1);
                let to_yield_after_text = InlineEvent::Raw(start..cursor.value());
                return Some((start, Some(to_yield_after_text)));
            } else if (is_hex && !char.is_ascii_hexdigit()) || (!is_hex && !char.is_ascii_digit()) {
                break;
            }

            cursor.move_forward(1);
        }
        None
    }
}

mod leaf {
    use super::*;

    pub mod ref_link {
        use super::*;

        pub fn process_potential(
            input: &[u8],
            cursor: &mut Cursor,
        ) -> Option<(usize, Option<InlineEvent>)> {
            let maybe_text_end = cursor.value();
            cursor.move_forward(">>".len());
            let start = cursor.value();

            let ref_link_content =
                leaf::ref_link::advance_until_potential_content_ends(input, cursor);
            if let Some(()) = ref_link_content {
                let to_yield_after_text = InlineEvent::RefLink(start..cursor.value());
                Some((maybe_text_end, Some(to_yield_after_text)))
            } else {
                None
            }
        }

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

        /// TODO: 支持多行。
        ///
        /// NOTE: 为什么 `dicexp` 不能像 `code_span` 那样相对简单地支持多行：
        /// - `code_span` 基于原生的 HTML 元素 `<code/>`，内容可以直接作为其子节
        ///   点，因此设计成了用两个事件（`EnterCodeSpan`、`Exit`）来包围内容。
        ///   
        /// - `dicexp` 基于自定义元素，需要以属性（attribute）传递内容，因此设计
        ///   为了只用一个事件（`Dicexp`）。
        ///
        /// 后者的事件只能传递一整块范围的内容，而多行内容的范围并不连续，因此无
        /// 法传递。未来可能会让 `dicexp` 的事件也以 Enter/Exit 的形式表示，在那
        /// 之前 dicexp 无法支持多行。
        pub fn process(input: &[u8], cursor: &mut Cursor) -> (usize, Option<InlineEvent>) {
            let text_end = cursor.value();

            cursor.move_forward("[=".len());
            let content = leaf::dicexp::advance_until_ends(input, cursor);
            let to_yield_after_text = InlineEvent::Dicexp(content);

            (text_end, Some(to_yield_after_text))
        }

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

        pub fn process<TInlineStack: Stack<StackEntry>>(
            input: &[u8],
            cursor: &mut Cursor,
            inner: &mut ParserInner<TInlineStack>,
        ) -> (usize, Option<InlineEvent>) {
            let text_end = cursor.value();

            let backticks =
                "`".len() + count_continuous_character(input, m!('`'), cursor.value() + "[`".len());
            cursor.move_forward("[".len() + backticks);

            if input.get(cursor.value()) == Some(&b' ') {
                cursor.move_forward(1);
            }

            let top_leaf = TopLeafCodeSpan { backticks };
            let ev = top_leaf.make_enter_event();
            inner.stack.push_top_leaf(top_leaf.into());

            (text_end, Some(ev))
        }

        pub fn parse_content_and_process<TInlineStack: Stack<StackEntry>>(
            input: &[u8],
            cursor: &mut Cursor,
            inner: &mut ParserInner<TInlineStack>,
            top_leaf: TopLeafCodeSpan,
        ) -> crate::Result<Tym<2>> {
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

                let mut content_end = cursor.value();
                if cursor.value() > start && input.get(cursor.value() - 1) == Some(&b' ') {
                    content_end -= 1;
                }

                let tym_a = yield_text_if_not_empty(start, content_end, inner);

                cursor.move_forward(top_leaf.backticks + "]".len());
                let tym_b = inner.r#yield(top_leaf.make_exit_event());

                return Ok(tym_a.add(tym_b));
            }

            inner.stack.push_top_leaf(top_leaf.into());
            let tym = inner.r#yield(InlineEvent::Text(start..cursor.value()));
            Ok(tym.into())
        }
    }
}
