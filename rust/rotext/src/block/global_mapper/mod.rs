mod tests;

use crate::{
    common::Range,
    events::{GlobalEvent, NewLine, VerbatimEscaping},
    global,
};

/// 用于将产出 [global::Event] 的流转化为便于 [Parser] 处理的流。
pub struct GlobalEventStreamMapper<'a> {
    input: &'a [u8],
    stream: global::Parser<'a>,

    remain: Option<RemainUnparsed>,
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
    /// 换行。
    NewLine(NewLine),
    /// 文本。
    VerbatimEscaping(VerbatimEscaping),
}
impl Mapped {
    pub fn is_new_line(&self) -> bool {
        matches!(self, Mapped::NewLine(_))
    }
}

impl<'a> GlobalEventStreamMapper<'a> {
    pub fn new(input: &'a [u8], stream: global::Parser<'a>) -> GlobalEventStreamMapper<'a> {
        GlobalEventStreamMapper {
            input,
            stream,
            remain: None,
        }
    }

    pub fn next(&mut self) -> Option<Mapped> {
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

            let next = self.stream.next();

            if let Some(GlobalEvent::Unparsed(content)) = next {
                self.remain = Some(RemainUnparsed {
                    content,
                    next_offset: 0,
                });
                continue;
            }

            match next? {
                GlobalEvent::Unparsed(_) => unreachable!(),
                GlobalEvent::VerbatimEscaping(verbatim_escaping) => {
                    return Some(Mapped::VerbatimEscaping(verbatim_escaping));
                }
                GlobalEvent::NewLine(new_line) => return Some(Mapped::NewLine(new_line)),
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
