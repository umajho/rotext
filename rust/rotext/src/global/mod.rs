mod events;

pub use events::Event;

use crate::common::Range;

pub struct Parser<'a> {
    input: &'a [u8],
    cursor: usize,
    state: State,
}

enum State {
    Ended,
    Normal,
    InVerbatimEscaping { backticks: usize },
    InComment,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a [u8], cursor: usize) -> Parser<'a> {
        Parser {
            input,
            cursor,
            state: State::Normal,
        }
    }

    pub fn next(&mut self) -> Option<Event> {
        loop {
            let result = match self.state {
                State::Ended => {
                    break None;
                }
                State::Normal => self.scan_normal(),
                State::InVerbatimEscaping { backticks } => self.scan_verbatim_escaping(backticks),
                State::InComment => self.scan_comment(),
            };
            if result.is_none() {
                // 除了已经解析结束（`State::Ended`）外，`None` 还用于表示这次扫
                // 描没有产出事件（如注释、两个相邻的特殊语法之间的一般内容）。
                // 这时，直接进行下一次扫描。
                continue;
            }
            break result;
        }
    }

    fn scan_normal(&mut self) -> Option<Event> {
        let mut offset = 0;
        loop {
            let index = self.cursor + offset;
            match self.input.get(index) {
                None => {
                    // 在一般情况下到达输入结尾，完成扫描并结束解析。
                    self.state = State::Ended;
                    break if offset != 0 {
                        self.produce_undetermined(offset)
                    } else {
                        None
                    };
                }
                Some(b'\r') => {
                    let ret = self.produce_undetermined(offset);
                    self.cursor += "\r".len();
                    break ret;
                }
                Some(b'<') => match self.input.get(index + 1) {
                    Some(b'`') => {
                        // 在一般情况下遇到 “<`”，完成扫描并开启逐字文本转义。
                        let ret = self.produce_undetermined(offset);

                        let backticks = {
                            let start_index = self.cursor + 2;
                            "`".len() + count_continuous_backticks(self.input, start_index, None)
                        };
                        self.state = State::InVerbatimEscaping { backticks };
                        self.cursor += "<".len() + backticks;

                        break ret;
                    }
                    Some(b'%') => {
                        // 在一般情况下遇到 “<%”，完成扫描并开启注释。
                        let ret = self.produce_undetermined(offset);

                        self.state = State::InComment;
                        self.cursor += "<%".len();

                        break ret;
                    }
                    None => {
                        // 在一般情况下遇到 “<” 后到达输入结尾，完成扫描并结束解
                        // 析。
                        let ret = self.produce_undetermined(offset + "<".len());
                        self.state = State::Ended;
                        break ret;
                    }
                    _ => {
                        // 在一般情况下遇到 “<” 后遇到了不会与 “<” 组合而具有特
                        // 殊含义的字符，继续扫描。
                        offset += "<".len();
                    }
                },
                _ => {
                    // 在一般情况下遇到不会有特殊含义的字符，继续扫描。
                    offset += 1;
                }
            }
        }
    }

    fn scan_verbatim_escaping(&mut self, backticks: usize) -> Option<Event> {
        let mut offset = 0;
        loop {
            let index = self.cursor + offset;

            match self.input.get(index) {
                None => {
                    // 在逐字文本转义中到达结尾，完成扫描并结束解析。
                    self.state = State::Ended;
                    break self.produce_verbatim_escaping(backticks, offset, false);
                }
                Some(b'`') => {
                    // 在逐字文本转义中遇到 “`”，可能是逐字文本转义闭合部分的开
                    // 始。

                    offset += "`".len();

                    let second_backtick_index = self.cursor + offset;
                    let result = match_verbatim_escaping_closing_part(
                        self.input,
                        second_backtick_index,
                        backticks,
                    );

                    offset += result.advancing;
                    if result.is_matched {
                        let ret = self.produce_verbatim_escaping(
                            backticks,
                            offset - backticks - ">".len(),
                            true,
                        );
                        self.state = State::Normal;
                        break ret;
                    }
                }
                _ => {
                    offset += 1;
                }
            }
        }
    }

    fn scan_comment(&mut self) -> Option<Event> {
        // 深度为 0 代表只有一层注释。
        let mut depth = 0;
        // 如果位于逐字文本转义之中，本变量为包裹着 backtick 数量的 `Some`。
        let mut verbatim_escaping: Option<usize> = None;

        let mut offset = 0;
        loop {
            let index = self.cursor + offset;

            let Some(&char) = self.input.get(index) else {
                // 在注释中到达结尾，完成扫描并结束解析。
                self.state = State::Ended;
                break self.produce_comment(offset, false);
            };

            if let Some(expected_backticks) = verbatim_escaping {
                // 在注释中的在逐字文本转义之内。

                offset += 1;
                if char != b'`' {
                    // 不是 “`”，则不可能是逐字文本转义闭合部分的开始。
                    continue;
                }

                let second_backtick_index = self.cursor + offset;
                let result = match_verbatim_escaping_closing_part(
                    self.input,
                    second_backtick_index,
                    expected_backticks,
                );

                offset += result.advancing;
                if result.is_matched {
                    verbatim_escaping = None;
                }
            }

            match char {
                b'<' => match self.input.get(index + 1) {
                    Some(b'`') => {
                        // 在注释中遇到 “<`”，逐字文本转义深入一层。
                        offset += "<`".len();
                        let backticks = {
                            let start_index = self.cursor + offset;
                            "`".len() + count_continuous_backticks(self.input, start_index, None)
                        };
                        verbatim_escaping = Some(backticks);
                        offset = offset - "`".len() + backticks;
                    }
                    Some(b'%') => {
                        // 在注释中遇到 “<%”。
                        offset += "<%".len();
                        depth += 1;
                    }
                    _ => {
                        // 在注释中遇到 “<” 后遇到了不会与 “<” 组合而具有特殊含
                        // 义的字符，继续扫描。
                        offset += "<".len();
                    }
                },
                b'%' => {
                    // 在注释中遇到 “%”，可能是注释的闭合部分。

                    offset += "%".len();

                    if self.input.get(self.cursor + offset) != Some(&b'>') {
                        continue;
                    }
                    offset += ">".len();
                    if depth > 0 {
                        depth -= 1;
                    } else {
                        let ret = self.produce_comment(offset - "%>".len(), true);
                        self.state = State::Normal;
                        break ret;
                    }
                }
                _ => {
                    // 在注释中遇到不会有特殊含义的字符，继续扫描。
                    offset += 1;
                }
            }
        }
    }

    /// 产出以 `self.cursor` 开始，长度为 `content_length` 的
    /// [Event::Undetermined]，这之后，`self.cursor` 移至下个 Event 的开始。
    fn produce_undetermined(&mut self, length: usize) -> Option<Event> {
        if length == 0 {
            return None;
        }

        let ret = Event::Undetermined(Range::new(self.cursor, length));
        self.cursor += length;
        Some(ret)
    }

    fn produce_verbatim_escaping(
        &mut self,
        backtick_length: usize,
        content_length: usize,
        is_closed_normally: bool,
    ) -> Option<Event> {
        let ret = Event::VerbatimEscaping {
            content: Range::new(self.cursor, content_length),
            is_closed_forcedly: !is_closed_normally,
        };
        self.cursor += content_length;
        if is_closed_normally {
            self.cursor += backtick_length + ">".len();
        }
        Some(ret)
    }

    fn produce_comment(
        &mut self,
        content_length: usize,
        is_closed_normally: bool,
    ) -> Option<Event> {
        let ret = Event::Comment {
            content: Range::new(self.cursor, content_length),
            is_closed_forcedly: !is_closed_normally,
        };
        self.cursor += content_length;
        if is_closed_normally {
            self.cursor += "%>".len();
        }
        Some(ret)
    }
}

impl<'a> Iterator for Parser<'a> {
    type Item = Event;

    fn next(&mut self) -> Option<Self::Item> {
        self.next()
    }
}

struct VerbatimEscapingClosingPartMatchResult {
    is_matched: bool,
    advancing: usize,
}

fn match_verbatim_escaping_closing_part(
    input: &[u8],
    second_backtick_index: usize,
    expected_backticks: usize,
) -> VerbatimEscapingClosingPartMatchResult {
    let expected_gt_index = second_backtick_index - "`".len() + expected_backticks;
    if input.get(expected_gt_index) != Some(&b'>') {
        // 在数 “`” 之前先确认一下对应位置有没有 “>”，没有的话就不用数了，逐字文
        // 本转义的闭合部分肯定不是从这里开始。
        return VerbatimEscapingClosingPartMatchResult {
            is_matched: false,
            advancing: 0,
        };
    }

    let got_backticks = {
        "`".len()
            + count_continuous_backticks(input, second_backtick_index, Some(expected_backticks))
    };

    if got_backticks == expected_backticks {
        VerbatimEscapingClosingPartMatchResult {
            is_matched: true,
            advancing: got_backticks - "`".len() + ">".len(),
        }
    } else {
        // 没有匹配到足够数量的 “`”，于是都视为不存在特殊含义。
        VerbatimEscapingClosingPartMatchResult {
            is_matched: false,
            advancing: got_backticks - "`".len(),
        }
    }
}

/// 从 `start_index` 起（含开始）计数连续的 “`” 的数量。
fn count_continuous_backticks(input: &[u8], start_index: usize, max: Option<usize>) -> usize {
    if max == Some(0) {
        return 0;
    }

    let mut count = 0;
    while input.get(start_index + count) == Some(&b'`') {
        count += 1;
        if max.is_some_and(|max| count == max) {
            break;
        }
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    use std::{collections::HashSet, time};

    use crate::events::EventType;

    type EventCase = (
        EventType,
        Option<&'static str>,
        Option<HashSet<&'static str>>,
    );

    #[rstest]
    // ## 无特殊语法
    #[case("", vec![])]
    #[case("Hello, world!", vec![
        (EventType::Undetermined, Some("Hello, world!"), None)])]
    // ### 无视 CR
    #[case("\r", vec![])]
    #[case("Left\rRight", vec![
        (EventType::Undetermined, Some("Left"), None),
        (EventType::Undetermined, Some("Right"), None)])]
    // ## 逐字文本转义
    #[case("<` … `>", vec![
        (EventType::VerbatimEscaping, Some(" … "), None)])]
    #[case("<`` `> ``>", vec![
        (EventType::VerbatimEscaping, Some(" `> "), None)])]
    #[case("<` A `><` B `>", vec![
        (EventType::VerbatimEscaping, Some(" A "), None),
        (EventType::VerbatimEscaping, Some(" B "), None)])]
    #[case("Left<` … `>Right", vec![
        (EventType::Undetermined, Some("Left"), None),
        (EventType::VerbatimEscaping, Some(" … "), None),
        (EventType::Undetermined, Some("Right"), None)])]
    #[case("<` `> `>", vec![
        (EventType::VerbatimEscaping, Some(" "), None),
        (EventType::Undetermined, Some(" `>"), None)])]
    #[case("<` <` `>", vec![
        (EventType::VerbatimEscaping, Some(" <` "), None)])]
    #[case("Foo<`Bar", vec![
        (EventType::Undetermined, Some("Foo"), None),
        (EventType::VerbatimEscaping, Some("Bar"), Some(HashSet::from(["F"])))])]
    // ## 注释
    #[case("<% … %>", vec![
        (EventType::Comment, Some(" … "), None)])]
    #[case("<% A %><% B %>", vec![
        (EventType::Comment, Some(" A "), None),
        (EventType::Comment, Some(" B "), None)])]
    #[case("Left<%%>Right", vec![
        (EventType::Undetermined, Some("Left"), None),
        (EventType::Comment, Some(""), None),
        (EventType::Undetermined, Some("Right"), None)])]
    #[case("Foo<%Bar", vec![
        (EventType::Undetermined, Some("Foo"), None),
        (EventType::Comment, Some("Bar"), Some(HashSet::from(["F"])))])]
    #[case("Foo<%Bar<%Baz%>Qux", vec![
        (EventType::Undetermined, Some("Foo"), None),
        (EventType::Comment, Some("Bar<%Baz%>Qux"), Some(HashSet::from(["F"])))])]
    #[case("Foo%>Bar", vec![
        (EventType::Undetermined, Some("Foo%>Bar"), None)])]
    #[case("Foo<%Bar%>Baz%>Qux", vec![
        (EventType::Undetermined, Some("Foo"), None),
        (EventType::Comment, Some("Bar"), None),
        (EventType::Undetermined, Some("Baz%>Qux"), None)])]
    #[case("0 <% 1 <% 2 %> 1 %> 0", vec![
        (EventType::Undetermined, Some("0 "), None),
        (EventType::Comment, Some(" 1 <% 2 %> 1 "), None),
        (EventType::Undetermined, Some(" 0"), None)])]
    // ### 注释中的逐字文本转义
    #[case("0 <% 1 <`<%`> 2 %> 1 %> 0", vec![
        (EventType::Undetermined, Some("0 "), None),
        (EventType::Comment, Some(" 1 <`<%`> 2 "), None),
        (EventType::Undetermined, Some(" 1 %> 0"), None)])]
    #[case("0 <% 1 <% 2 <`%>`> 1 %> 0", vec![
        (EventType::Undetermined, Some("0 "), None),
        (EventType::Comment, Some(" 1 <% 2 <`%>`> 1 %> 0"), Some(HashSet::from(["F"])))])]
    #[timeout(time::Duration::from_secs(1))]
    fn it_works(#[case] input: &str, #[case] expected: Vec<EventCase>) {
        let parser = Parser::new(input.as_bytes(), 0);
        let actual: Vec<_> = parser
            .map(|ev| -> EventCase {
                (
                    EventType::from(ev.discriminant()),
                    ev.content(input.as_bytes())
                        .map(|s| -> &'static str { s.leak() }),
                    ev.assertion_flags(),
                )
            })
            .collect();

        assert_eq!(expected, actual);
    }
}
