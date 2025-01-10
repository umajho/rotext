mod parser_inner;
mod stack_wrapper;
mod types;

#[cfg(test)]
mod tests;

pub use stack_wrapper::StackEntry;

use std::ops::Range;

use crate::{
    events::{ev, NewLine},
    utils::internal::peekable::Peekable,
    Event,
};
use parser_inner::{ParserInner, ToSkipInputEvents};
use stack_wrapper::{TopLeaf, TopLeafCodeSpan};
use types::{Cursor, YieldContext};

use crate::{
    common::{is_valid_character_in_name, m},
    events::VerbatimEscaping,
    types::{Tym, TYM_UNIT},
    utils::{
        internal::string::{
            count_continuous_character, count_continuous_character_with_maximum, is_whitespace,
        },
        stack::Stack,
    },
};

pub struct Parser<'a, TInlineStack: Stack<StackEntry>> {
    full_input: &'a [u8],

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
    pub fn new(full_input: &'a [u8]) -> Self {
        Self {
            full_input,
            state: State::Idle,
            inner: ParserInner::new(),
        }
    }

    /// `event_stream` 的迭代对象是属于 `InlineInput` 分组的事件。返回的事件属于
    /// `Inline` 分组。
    pub fn next(
        &mut self,
        event_stream: &mut Peekable<2, impl Iterator<Item = Event>>,
    ) -> Option<crate::Result<Event>> {
        loop {
            if let Some(ev) = self.inner.pop_to_be_yielded() {
                break Some(Ok(ev));
            }

            if let Some(end_entry) = &self.inner.to_exit_until_popped_entry_from_stack {
                let entry = self.inner.stack.pop_entry().unwrap();
                if &entry == end_entry {
                    self.inner.to_exit_until_popped_entry_from_stack = None;
                }
                break Some(Ok(ev!(Inline, ExitInline)));
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

                    if self.inner.to_skip_input.count > 0 {
                        self.inner.to_skip_input.count -= 1;
                        if self.inner.to_skip_input.count == 0 {
                            if let Some(cursor_value) = self.inner.to_skip_input.cursor_value.take()
                            {
                                let ev!(InlineInput, __Unparsed(content)) = next else {
                                    unreachable!()
                                };
                                let input = &self.full_input[..content.end];
                                let cursor = Cursor::new(cursor_value);
                                self.state = State::Parsing { input, cursor };
                            }
                        }
                        continue;
                    }

                    let to_yield = #[rotext_internal_macros::ensure_cases_for_event(
                        prefix = Event,
                        group = InlineInput,
                    )]
                    // NOTE: rust-analyzer 会错误地认为这里的 `match` 没有覆盖到
                    // 全部分支，实际上并不存在问题。
                    match next {
                        Event::__Unparsed(content) => {
                            let input = &self.full_input[..content.end];
                            let cursor = Cursor::new(content.start);
                            self.state = State::Parsing { input, cursor };
                            continue;
                        }
                        Event::VerbatimEscaping(verbatim_escaping) => {
                            ev!(Inline, VerbatimEscaping(verbatim_escaping))
                        }
                        Event::NewLine(new_line) => ev!(Inline, NewLine(new_line)),
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

    /// `event_stream` 的迭代对象是属于 `InlineInput` 分组的事件。
    fn parse(
        input: &[u8],
        cursor: &mut Cursor,
        inner: &mut ParserInner<TInlineStack>,
        event_stream: &mut Peekable<2, impl Iterator<Item = Event>>,
    ) -> crate::Result<Tym<4>> {
        match inner.stack.pop_top_leaf() {
            None => Self::parse_normal(input, cursor, inner, event_stream),
            Some(TopLeaf::CodeSpan(top_leaf)) => {
                leaf::code_span::parse_content_and_process(input, cursor, inner, top_leaf)
                    .map(|tym| tym.into())
            }
        }
    }

    /// `event_stream` 的迭代对象是属于 `InlineInput` 分组的事件。
    fn parse_normal(
        input: &[u8],
        cursor: &mut Cursor,
        inner: &mut ParserInner<TInlineStack>,
        event_stream: &mut Peekable<2, impl Iterator<Item = Event>>,
    ) -> crate::Result<Tym<4>> {
        let end_condition = inner.stack.make_end_condition();

        let text_start = cursor.value();
        // `to_yield_after_text` 是属于 `Inline` 分组的事件。
        let (text_end, to_yield_after_text): (usize, Option<Event>) = loop {
            let Some(char) = input.get(cursor.value()) else {
                break (cursor.value(), None);
            };

            match char {
                m!('\\') if cursor.value() == input.len() - 1 => {
                    let Some(ev!(InlineInput, NewLine(new_line))) = event_stream.peek(0) else {
                        cursor.move_forward(1);
                        continue;
                    };
                    let new_line = new_line.clone();
                    break special::process_hard_break_mark(input, cursor, inner, new_line);
                }
                m!('_') if cursor.value() == input.len() - 1 => {
                    let Some(ev!(InlineInput, NewLine(_))) = event_stream.peek(0) else {
                        cursor.move_forward(1);
                        continue;
                    };
                    break special::process_lines_joint_mark(input, cursor, inner);
                }
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
                        let to_yield_after_text = ev!(Inline, EnterStrong);

                        break (text_end, Some(to_yield_after_text));
                    }
                    Some(m!('~')) => {
                        let text_end = cursor.value();

                        cursor.move_forward("[~".len());
                        inner.stack.push_entry(StackEntry::Strikethrough)?;
                        let to_yield_after_text = ev!(Inline, EnterStrikethrough);

                        break (text_end, Some(to_yield_after_text));
                    }
                    Some(m!('[')) => {
                        match leaf::wiki_link::process_and_yield_potential(
                            input,
                            text_start,
                            cursor,
                            inner,
                            event_stream,
                        )? {
                            Some(tym) => {
                                return Ok(tym);
                            }
                            None => continue,
                        }
                    }
                    Some(_) => {
                        cursor.move_forward(1);
                        continue;
                    }
                },
                &char => {
                    if let Some(entry_to_be_popped_until) =
                        end_condition.test(char, input.get(cursor.value() + 1).copied())
                    {
                        let text_end = cursor.value();
                        cursor.move_forward(2);
                        inner.to_exit_until_popped_entry_from_stack =
                            Some(entry_to_be_popped_until);
                        break (text_end, None);
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

        Ok(tym_a.add(tym_b).into())
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
                StackEntry::Strong | StackEntry::Strikethrough | StackEntry::WikiLink => {
                    inner.r#yield(ev!(Inline, ExitInline))
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
        inner.r#yield(ev!(Inline, Text(start..end)))
    } else {
        TYM_UNIT.into()
    }
}

mod special {

    use super::*;

    /// 返回的事件属于 `Inline` 分组。
    pub fn process_hard_break_mark<TInlineStack: Stack<StackEntry>>(
        _input: &[u8],
        cursor: &mut Cursor,
        inner: &mut ParserInner<TInlineStack>,
        new_line: NewLine,
    ) -> (usize, Option<Event>) {
        let text_end = cursor.value();
        cursor.move_forward(1);
        inner.to_skip_input = ToSkipInputEvents::new_one();
        let to_yield_after_text = ev!(Inline, NewLine(new_line));

        (text_end, Some(to_yield_after_text))
    }

    /// 返回的事件属于 `Inline` 分组。
    pub fn process_lines_joint_mark<TInlineStack: Stack<StackEntry>>(
        _input: &[u8],
        cursor: &mut Cursor,
        inner: &mut ParserInner<TInlineStack>,
    ) -> (usize, Option<Event>) {
        let text_end = cursor.value();
        cursor.move_forward(1);
        inner.to_skip_input = ToSkipInputEvents::new_one();

        (text_end, None)
    }

    /// 返回的事件属于 `Inline` 分组。
    pub fn process_potential_numeric_character_reference(
        input: &[u8],
        cursor: &mut Cursor,
    ) -> Option<(usize, Option<Event>)> {
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
                let to_yield_after_text = ev!(Inline, Raw(start..cursor.value()));
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

        /// 返回的事件属于 `Inline` 分组。
        pub fn process_potential(
            input: &[u8],
            cursor: &mut Cursor,
        ) -> Option<(usize, Option<Event>)> {
            let maybe_text_end = cursor.value();
            cursor.move_forward(">>".len());
            let start = cursor.value();

            let ref_link_content = advance_until_potential_content_ends(input, cursor);
            if let Some(()) = ref_link_content {
                let to_yield_after_text = ev!(Inline, RefLink(start..cursor.value()));
                Some((maybe_text_end, Some(to_yield_after_text)))
            } else {
                cursor.set_value(cursor.value() - 1);
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
            if *char == b'#'
                && input
                    .get(cursor.value() + 1)
                    .is_some_and(|c| c.is_ascii_digit())
            {
                advance_until_potential_floor_address_content_ends(input, cursor)
            } else if char.is_ascii_alphabetic() {
                advance_until_potential_absolute_address_content_ends(input, cursor)
            } else {
                None
            }
        }

        fn advance_until_potential_floor_address_content_ends(
            input: &[u8],
            cursor: &mut Cursor,
        ) -> Option<()> {
            cursor.move_forward(2);

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

        fn advance_until_potential_absolute_address_content_ends(
            input: &[u8],
            cursor: &mut Cursor,
        ) -> Option<()> {
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

        /// 返回的事件属于 `Inline` 分组。
        ///
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
        pub fn process(input: &[u8], cursor: &mut Cursor) -> (usize, Option<Event>) {
            let text_end = cursor.value();

            cursor.move_forward("[=".len());
            let content = leaf::dicexp::advance_until_ends(input, cursor);
            let to_yield_after_text = ev!(Inline, Dicexp(content));

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

        /// 返回的事件属于 `Inline` 分组。
        pub fn process<TInlineStack: Stack<StackEntry>>(
            input: &[u8],
            cursor: &mut Cursor,
            inner: &mut ParserInner<TInlineStack>,
        ) -> (usize, Option<Event>) {
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
        ) -> crate::Result<Tym<3>> {
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

                return Ok(tym_a.add(tym_b).into());
            }

            inner.stack.push_top_leaf(top_leaf.into());
            let tym = inner.r#yield(ev!(Inline, Text(start..cursor.value())));
            Ok(tym.into())
        }
    }

    pub mod wiki_link {

        use super::*;

        /// `event_stream` 的迭代对象是属于 `InlineInput` 分组的事件。
        pub fn process_and_yield_potential<TInlineStack: Stack<StackEntry>>(
            input: &[u8],
            text_start: usize,
            cursor: &mut Cursor,
            inner: &mut ParserInner<TInlineStack>,
            event_stream: &mut Peekable<2, impl Iterator<Item = Event>>,
        ) -> crate::Result<Option<Tym<4>>> {
            let maybe_text_end = cursor.value();
            cursor.move_forward("[[".len());

            let (address, address_ev, indicator) = if let (Some(slot_content), after_slot) =
                parse_first_slot_for_non_verbatim(input, cursor)
            {
                let AfterSlot::Indicator {
                    indicator,
                    index_after_indicator,
                } = after_slot
                else {
                    // 有内容（标题）但没找到指示标记时，不视为Wiki链接。
                    // 如：`[[f<`oo`>]]`、`[[f\noo]]` 都不被视为Wiki链接。
                    return Ok(None);
                };

                cursor.set_value(index_after_indicator);
                let address = slot_content.clone();
                let address_ev = ev!(Inline, Text(slot_content));

                (address, address_ev, indicator)
            } else {
                let Some((
                    slot,
                    AfterSlot::Indicator {
                        indicator,
                        index_after_indicator,
                    },
                )) = parse_first_slot_for_verbatim(input, event_stream)
                else {
                    return Ok(None);
                };

                // 跳过当前正在处理的事件（即以 “[[” 结尾的事件）以及作为第一个槽位的逐字转译的事件。
                // 由于完成跳过后会设置游标，这里不用再用 `cursor.set_value` 来设置游标。
                inner.to_skip_input = ToSkipInputEvents {
                    count: 2,
                    cursor_value: Some(index_after_indicator),
                };

                let address = slot.content.clone();
                let address_ev = ev!(Inline, VerbatimEscaping(slot));

                (address, address_ev, indicator)
            };

            let tym_a = process_first_slot(text_start, maybe_text_end, inner, address)?;
            let tym_b = process_indicator(inner, address_ev, indicator)?;
            Ok(Some(tym_a.add(tym_b)))
        }

        fn process_first_slot<TInlineStack: Stack<StackEntry>>(
            text_start: usize,
            text_end: usize,
            inner: &mut ParserInner<TInlineStack>,
            address: Range<usize>,
        ) -> crate::Result<Tym<2>> {
            let tym_a = yield_text_if_not_empty(text_start, text_end, inner);
            let tym_b = inner.r#yield(ev!(Inline, EnterWikiLink(address.clone())));
            Ok(tym_a.add(tym_b))
        }

        /// `address_ev` 是属于 Inline 分组的事件，其具体应该是
        /// [Event::Text] 或  [Event::VerbatimEscaping]。
        fn process_indicator<TInlineStack: Stack<StackEntry>>(
            inner: &mut ParserInner<TInlineStack>,
            address_ev: Event,
            indicator: Indicator,
        ) -> crate::Result<Tym<2>> {
            let tym = match indicator {
                Indicator::Closing => {
                    let tym_c1 = inner.r#yield(address_ev);
                    let tym_c2 = inner.r#yield(ev!(Inline, ExitInline));
                    tym_c1.add(tym_c2)
                }
                Indicator::Separator => {
                    inner.stack.push_entry(StackEntry::WikiLink)?;
                    TYM_UNIT.into()
                }
            };
            Ok(tym)
        }

        #[derive(Debug)]
        enum AfterSlot {
            /// 第二个值是第一个值代表内容之后的索引（第二个 `]` 或 `|` 之后的索
            /// 引）。
            Indicator {
                indicator: Indicator,
                index_after_indicator: usize,
            },
            Nothing,
        }

        #[derive(Debug)]
        enum Indicator {
            /// `]]`。
            Closing,
            /// `|`。
            Separator,
        }

        /// 解析第一个槽位的内容。（不将逐字内容视为有效的槽位内容。）
        ///
        /// NOTE: cursor 只移动到第一处非空白字符之前。
        fn parse_first_slot_for_non_verbatim(
            input: &[u8],
            cursor: &mut Cursor,
        ) -> (Option<Range<usize>>, AfterSlot) {
            #[derive(Debug)]
            struct MiniInclusiveRange {
                start: usize,
                end: usize,
            }
            impl From<MiniInclusiveRange> for Range<usize> {
                fn from(range: MiniInclusiveRange) -> Self {
                    range.start..(range.end + 1)
                }
            }

            let mut slot_content: Option<MiniInclusiveRange> = None;
            let mut after_slot = AfterSlot::Nothing;

            while let Some(&char) = input.get(cursor.value()) {
                if is_whitespace!(char) {
                    cursor.move_forward(1);
                } else {
                    break;
                }
            }

            for i in cursor.value()..input.len() {
                // SAFETY: `cursor.value()` <= `i` < `input.len()`.
                let char = unsafe { input.get_unchecked(i) };
                match char {
                    char if is_whitespace!(char) => continue,
                    m!(']') if input.get(i + 1) == Some(&m!(']')) => {
                        if slot_content.is_some() {
                            after_slot = AfterSlot::Indicator {
                                indicator: Indicator::Closing,
                                index_after_indicator: i + 2,
                            };
                            return (slot_content.map(|r| r.into()), after_slot);
                        } else {
                            return (None, AfterSlot::Nothing);
                        }
                    }
                    m!('|') => {
                        if slot_content.is_some() {
                            after_slot = AfterSlot::Indicator {
                                indicator: Indicator::Separator,
                                index_after_indicator: i + 1,
                            };
                            return (slot_content.map(|r| r.into()), after_slot);
                        } else {
                            return (None, AfterSlot::Nothing);
                        }
                    }
                    char if !is_valid_character_in_name(*char) => {
                        return (None, AfterSlot::Nothing);
                    }
                    _ => match slot_content {
                        Some(ref mut slot_content) => slot_content.end = i,
                        None => slot_content = Some(MiniInclusiveRange { start: i, end: i }),
                    },
                }
            }

            (slot_content.map(|r| r.into()), after_slot)
        }

        fn parse_first_slot_for_verbatim(
            input: &[u8],
            event_stream: &mut Peekable<2, impl Iterator<Item = Event>>,
        ) -> Option<(VerbatimEscaping, AfterSlot)> {
            let slot = {
                let Some(ev!(InlineInput, VerbatimEscaping(ve))) = event_stream.peek(0) else {
                    return None;
                };
                ve.clone()
            };
            let after_slot = {
                let Some(ev!(InlineInput, __Unparsed(after_slot_content))) = event_stream.peek(1)
                else {
                    return None;
                };

                // SAFETY: `after_slot.end` < `full_input.len()`.
                // TODO: 不管怎样，现在的实现也太丑陋了，未来应该重构掉这里的 unsafe。
                let input =
                    unsafe { std::slice::from_raw_parts(input.as_ptr(), after_slot_content.end) };

                parse_leading_indicator(input, after_slot_content.start)
            };

            Some((slot, after_slot))
        }

        fn parse_leading_indicator(input: &[u8], mut i: usize) -> AfterSlot {
            while let Some(&char) = input.get(i) {
                if is_whitespace!(char) {
                    i += 1;
                } else {
                    break;
                }
            }

            if i == input.len() {
                return AfterSlot::Nothing;
            }
            debug_assert!(i < input.len());

            // SAFETY: `start_index` < `input.len()`.
            let char = unsafe { input.get_unchecked(i) };
            match char {
                m!(']') if input.get(i + 1) == Some(&m!(']')) => AfterSlot::Indicator {
                    indicator: Indicator::Closing,
                    index_after_indicator: i + 2,
                },
                m!('|') => AfterSlot::Indicator {
                    indicator: Indicator::Separator,
                    index_after_indicator: i + 1,
                },
                _ => AfterSlot::Nothing,
            }
        }
    }
}
