use crate::{common::Range, events::GlobalEvent};

pub struct Parser<'a> {
    input: &'a [u8],
    cursor: usize,
    state: State,
    deferred: Option<GlobalEvent>,
}

enum State {
    Ended,
    Normal,
    InVerbatimEscaping { backticks: usize },
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a [u8], cursor: usize) -> Parser<'a> {
        Parser {
            input,
            cursor,
            state: State::Normal,
            deferred: None,
        }
    }

    pub fn next(&mut self) -> Option<GlobalEvent> {
        loop {
            if self.deferred.is_some() {
                return self.deferred.take();
            }

            let result = match self.state {
                State::Ended => {
                    break None;
                }
                State::Normal => self.scan_normal(),
                State::InVerbatimEscaping { backticks } => self.scan_verbatim_escaping(backticks),
            };
            if result.is_none() {
                // 除了已经解析结束（`State::Ended`）外，`None` 还用于表示这次扫
                // 描没有产出事件（如两个紧邻的特殊语法之间的一般内容）。
                // 这时，直接进行下一次扫描。
                continue;
            }
            break result;
        }
    }

    fn scan_normal(&mut self) -> Option<GlobalEvent> {
        let mut offset = 0;
        loop {
            let index = self.cursor + offset;
            match self.input.get(index) {
                None => {
                    // 在一般情况下到达输入结尾，完成扫描并结束解析。
                    self.state = State::Ended;
                    break if offset != 0 {
                        self.produce_unparsed(offset)
                    } else {
                        None
                    };
                }
                Some(b'\r') => {
                    let ret = self.produce_unparsed(offset);
                    self.deferred = Some(GlobalEvent::CarriageReturn { index });
                    self.cursor += "\r".len();
                    break ret;
                }
                Some(b'\n') => {
                    let ret = self.produce_unparsed(offset);
                    self.deferred = Some(GlobalEvent::LineFeed { index });
                    self.cursor += "\n".len();
                    break ret;
                }
                Some(b'<') => match self.input.get(index + 1) {
                    Some(b'`') => {
                        // 在一般情况下遇到 “<`”，完成扫描并开启逐字文本转义。
                        let ret = self.produce_unparsed(offset);

                        let backticks = {
                            let start_index = self.cursor + 2;
                            "`".len() + count_continuous_backticks(self.input, start_index, None)
                        };
                        self.state = State::InVerbatimEscaping { backticks };
                        self.cursor += "<".len() + backticks;

                        break ret;
                    }
                    None => {
                        // 在一般情况下遇到 “<” 后到达输入结尾，完成扫描并结束解
                        // 析。
                        let ret = self.produce_unparsed(offset + "<".len());
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

    fn scan_verbatim_escaping(&mut self, backticks: usize) -> Option<GlobalEvent> {
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

    /// 产出以 `self.cursor` 开始，长度为 `content_length` 的
    /// [Event::Unparsed]，这之后，`self.cursor` 移至下个 Event 的开始。
    fn produce_unparsed(&mut self, length: usize) -> Option<GlobalEvent> {
        if length == 0 {
            return None;
        }

        let ret = GlobalEvent::Unparsed(Range::new(self.cursor, length));
        self.cursor += length;
        Some(ret)
    }

    fn produce_verbatim_escaping(
        &mut self,
        backtick_length: usize,
        content_length: usize,
        is_closed_normally: bool,
    ) -> Option<GlobalEvent> {
        let ret = GlobalEvent::VerbatimEscaping {
            content: Range::new(self.cursor, content_length),
            is_closed_forcedly: !is_closed_normally,
        };
        self.cursor += content_length;
        if is_closed_normally {
            self.cursor += backtick_length + ">".len();
        }
        Some(ret)
    }
}

impl<'a> Iterator for Parser<'a> {
    type Item = GlobalEvent;

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

    use crate::events::{Event, EventType};

    type EventCase<'a> = (EventType, Option<&'a str>, Option<HashSet<&'a str>>);

    #[rstest]
    // ## 无特殊语法
    #[case("", vec![])]
    #[case("Hello, world!", vec![
        (EventType::Unparsed, Some("Hello, world!"), None)])]
    // ### CR
    #[case("\r", vec![
        (EventType::CarriageReturn, None, None)])]
    #[case("Left\rRight", vec![
        (EventType::Unparsed, Some("Left"), None),
        (EventType::CarriageReturn, None, None),
        (EventType::Unparsed, Some("Right"), None)])]
    // ## 逐字文本转义
    #[case("<` … `>", vec![
        (EventType::VerbatimEscaping, Some(" … "), None)])]
    #[case("<`` `> ``>", vec![
        (EventType::VerbatimEscaping, Some(" `> "), None)])]
    #[case("<` line 1\nline 2 `>", vec![
        (EventType::VerbatimEscaping, Some(" line 1\nline 2 "), None)])]
    #[case("<` A `><` B `>", vec![
        (EventType::VerbatimEscaping, Some(" A "), None),
        (EventType::VerbatimEscaping, Some(" B "), None)])]
    #[case("Left<` … `>Right", vec![
        (EventType::Unparsed, Some("Left"), None),
        (EventType::VerbatimEscaping, Some(" … "), None),
        (EventType::Unparsed, Some("Right"), None)])]
    #[case("<` `> `>", vec![
        (EventType::VerbatimEscaping, Some(" "), None),
        (EventType::Unparsed, Some(" `>"), None)])]
    #[case("<` <` `>", vec![
        (EventType::VerbatimEscaping, Some(" <` "), None)])]
    #[case("Foo<`Bar", vec![
        (EventType::Unparsed, Some("Foo"), None),
        (EventType::VerbatimEscaping, Some("Bar"), Some(HashSet::from(["F"])))])]
    #[timeout(time::Duration::from_secs(1))]
    fn it_works(#[case] input: &str, #[case] expected: Vec<EventCase>) {
        let parser = Parser::new(input.as_bytes(), 0);
        let actual: Vec<_> = parser
            .map(|ev| -> EventCase {
                let ev: Event = ev.into();
                (
                    EventType::from(ev.discriminant()),
                    ev.content(input.as_bytes()),
                    ev.assertion_flags(),
                )
            })
            .collect();

        assert_eq!(expected, actual);
    }
}
