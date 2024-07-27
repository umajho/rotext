mod tests;

use crate::{
    common::Range,
    events::{GlobalEvent, NewLine, VerbatimEscaping},
};

pub struct Parser<'a> {
    input: &'a [u8],
    cursor: usize,
    state: State,
    current_line_number: usize,
    deferred: Option<GlobalEvent>,
}

enum State {
    Ended,
    Normal,
    InVerbatimEscaping { backticks: usize },
}

pub struct NewParserOptions {
    pub cursor: usize,
    pub current_line_number: usize,
}
impl Default for NewParserOptions {
    fn default() -> Self {
        Self {
            cursor: 0,
            current_line_number: 1,
        }
    }
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a [u8], opts: NewParserOptions) -> Parser<'a> {
        Parser {
            input,
            cursor: opts.cursor,
            state: State::Normal,
            current_line_number: opts.current_line_number,
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
            let ch = self.input.get(index);
            match ch {
                None => {
                    // 在一般情况下到达输入结尾，完成扫描并结束解析。
                    self.state = State::Ended;
                    break if offset != 0 {
                        self.produce_unparsed(offset)
                    } else {
                        None
                    };
                }
                Some(b'\r' | b'\n') => {
                    let ret = self.produce_unparsed(offset);
                    self.current_line_number += 1;
                    self.deferred = Some(GlobalEvent::NewLine(NewLine {
                        line_number_after: self.current_line_number,
                    }));
                    if ch == Some(&b'\r') && self.input.get(index + 1) == Some(&b'\n') {
                        self.cursor += "\r\n".len();
                    } else {
                        self.cursor += 1;
                    }
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
            let ch = self.input.get(index);
            match ch {
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
                Some(b'\r' | b'\n') => {
                    self.current_line_number += 1;
                    if ch == Some(&b'\r') && self.input.get(index + 1) == Some(&b'\n') {
                        offset += "\r\n".len();
                    } else {
                        offset += 1
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
        let (start, length) = {
            // 去掉开头与结尾可能存在的一个空格。
            let mut start = self.cursor;
            let mut length = content_length;

            if length >= 2 {
                if self.input[start] == b' ' {
                    start += 1;
                    length -= 1;
                }
                if is_closed_normally && self.input[start + length - 1] == b' ' {
                    length -= 1;
                }
            }

            (start, length)
        };

        let ret = GlobalEvent::VerbatimEscaping(VerbatimEscaping {
            content: Range::new(start, length),
            is_closed_forcedly: !is_closed_normally,
            line_number_after: self.current_line_number,
        });
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
