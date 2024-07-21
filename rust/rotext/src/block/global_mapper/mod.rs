mod tests;

use crate::{common::Range, events::GlobalEvent, global};

/// 用于将产出 [global::Event] 的流转化为便于 [Parser] 处理的流。
pub struct GlobalEventStreamMapper<'a> {
    input: &'a [u8],
    stream: global::Parser<'a>,

    deferred: Option<Deferred>,
    remain: Option<RemainUnparsed>,
    blank_at_line_beginning: Option<Range>,
}

enum Deferred {
    MappedToYield(Mapped),
    GlobalEventToMap(Option<GlobalEvent>),
}

#[derive(Debug)]
struct RemainUnparsed {
    content: Range,

    next_offset: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mapped {
    /// 对应索引的字符。
    CharAt(usize),
    /// 上个字符之后（索引 + 1）的那个字符。
    NextChar,
    /// LF 换行。
    LineFeed,
    /// 空白。
    BlankAtLineBeginning(Range),
    /// 文本。
    Text(Range),
}
impl Mapped {
    pub fn is_line_feed(&self) -> bool {
        matches!(self, Mapped::LineFeed)
    }
    pub fn is_blank_at_line_beginning(&self) -> bool {
        matches!(self, Mapped::BlankAtLineBeginning(..))
    }
}

impl<'a> GlobalEventStreamMapper<'a> {
    pub fn new(input: &'a [u8], stream: global::Parser<'a>) -> GlobalEventStreamMapper<'a> {
        GlobalEventStreamMapper {
            input,
            stream,
            deferred: None,
            remain: None,
            blank_at_line_beginning: Some(Range::new(0, 0)),
        }
    }

    pub fn next(&mut self) -> Option<Mapped> {
        let mut deferred = self.deferred.take();
        if let Some(Deferred::MappedToYield(mapped)) = deferred {
            self.deferred = None;
            return Some(mapped);
        }

        loop {
            if let Some(ref mut remain) = self.remain {
                // 先清掉剩余的。

                if let Some(mut blank) = self.blank_at_line_beginning {
                    while blank.length() < remain.content.length() {
                        let index = remain.content.start() + blank.length();
                        if self.input[index] != b' ' {
                            break;
                        }
                        blank.set_length(blank.length() + 1);
                    }
                    self.blank_at_line_beginning = None;

                    if blank.length() > 0 {
                        if remain.content.length() == blank.length() {
                            self.remain = None;
                        } else {
                            remain.content = Range::new(
                                remain.content.start() + blank.length(),
                                remain.content.length() - blank.length(),
                            )
                        }
                        return Some(Mapped::BlankAtLineBeginning(blank));
                    }
                }

                if remain.next_offset == remain.content.length() {
                    // 已经没有剩余的了。
                    self.remain = None;
                    continue;
                }

                let index = remain.content.start() + remain.next_offset;

                if remain.next_offset == 0 {
                    remain.next_offset += 1;
                    return Some(Mapped::CharAt(index));
                } else {
                    remain.next_offset += 1;
                    return Some(Mapped::NextChar);
                }
            }

            let next = {
                if let Some(Deferred::GlobalEventToMap(next)) = deferred.take() {
                    next
                } else {
                    self.stream.next()
                }
            };

            if let Some(GlobalEvent::Unparsed(content)) = next {
                self.remain = Some(RemainUnparsed {
                    content,
                    next_offset: 0,
                });
                continue;
            }

            if let Some(range) = self.blank_at_line_beginning.take() {
                if range.length() > 0 {
                    self.deferred = Some(Deferred::GlobalEventToMap(next));
                    return Some(Mapped::BlankAtLineBeginning(range));
                }
            }

            match next? {
                GlobalEvent::Unparsed(_) => unreachable!(),
                GlobalEvent::VerbatimEscaping {
                    content,
                    is_closed_forcedly,
                } => {
                    let (mut start, mut length) = (content.start(), content.length());
                    if length >= 2 {
                        if self.input[start] == b' ' {
                            start += 1;
                            length -= 1;
                        }
                        if !is_closed_forcedly && self.input[start + length - 1] == b' ' {
                            length -= 1;
                        }
                    }

                    let mapped_text = Mapped::Text(Range::new(start, length));
                    if let Some(range) = self.blank_at_line_beginning.take() {
                        if range.length() > 0 {
                            self.deferred = Some(Deferred::MappedToYield(mapped_text));
                            return Some(Mapped::BlankAtLineBeginning(range));
                        }
                    }
                    return Some(mapped_text);
                }
                GlobalEvent::CarriageReturn { index } => {
                    match self.stream.next() {
                        Some(GlobalEvent::LineFeed { index: lf_index }) => {
                            self.blank_at_line_beginning = Some(Range::new(lf_index + 1, 0));
                        }
                        None => {
                            // self.blank_at_line_beginning = Some(Range::new(index + 1, 0));
                            self.deferred = Some(Deferred::GlobalEventToMap(None));
                        }
                        otherwise => {
                            self.blank_at_line_beginning = Some(Range::new(index + 1, 0));
                            self.deferred = Some(Deferred::GlobalEventToMap(otherwise))
                        }
                    };
                    return Some(Mapped::LineFeed);
                }
                GlobalEvent::LineFeed { index } => {
                    self.blank_at_line_beginning = Some(Range::new(index + 1, 0));
                    return Some(Mapped::LineFeed);
                }
            }
        }
    }
}

impl<'a> Iterator for GlobalEventStreamMapper<'a> {
    type Item = Mapped;

    fn next(&mut self) -> Option<Self::Item> {
        self.next()
    }
}
