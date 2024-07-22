mod tests;

use crate::{common::Range, events::GlobalEvent, global};

/// 用于将产出 [global::Event] 的流转化为便于 [Parser] 处理的流。
pub struct GlobalEventStreamMapper<'a> {
    input: &'a [u8],
    stream: global::Parser<'a>,

    deferred: Option<Deferred>,
    remain: Option<RemainUnparsed>,
}

enum Deferred {
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
    /// 文本。
    Text(Range),
}
impl Mapped {
    pub fn is_line_feed(&self) -> bool {
        matches!(self, Mapped::LineFeed)
    }
}

impl<'a> GlobalEventStreamMapper<'a> {
    pub fn new(input: &'a [u8], stream: global::Parser<'a>) -> GlobalEventStreamMapper<'a> {
        GlobalEventStreamMapper {
            input,
            stream,
            deferred: None,
            remain: None,
        }
    }

    pub fn next(&mut self) -> Option<Mapped> {
        let mut deferred = self.deferred.take();

        loop {
            if let Some(ref mut remain) = self.remain {
                // 先清掉剩余的。

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
                    return Some(mapped_text);
                }
                GlobalEvent::CarriageReturn { .. } => {
                    match self.stream.next() {
                        Some(GlobalEvent::LineFeed { .. }) => {}
                        None => self.deferred = Some(Deferred::GlobalEventToMap(None)),
                        otherwise => self.deferred = Some(Deferred::GlobalEventToMap(otherwise)),
                    };
                    return Some(Mapped::LineFeed);
                }
                GlobalEvent::LineFeed { .. } => return Some(Mapped::LineFeed),
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
