use crate::{common::Range, events::EventType};

#[derive(Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum Event {
    /// 留给下个阶段解析。参见 [crate::global::Event::Unparsed]。
    Unparsed(Range) = EventType::Unparsed as u32,
    /// LF 换行。只在行内内容中产生。
    LineFeed = EventType::LineFeed as u32,

    /// 退出一层 “进入…”。
    Exit = EventType::Exit as u32,

    /// 进入段落。
    EnterParagraph = EventType::EnterParagraph as u32,
    /// 分割线
    ThematicBreak = EventType::ThematicBreak as u32,
    /// 代码块。
    EnterCodeBlock = EventType::EnterCodeBlock as u32,
    EnterCodeBlockMeta = EventType::EnterCodeBlockMeta as u32,
    EnterCodeBlockContent = EventType::EnterCodeBlockContent as u32,

    Text(Range) = EventType::Text as u32,
}

impl Event {
    #[cfg(test)]
    pub fn discriminant(&self) -> u32 {
        unsafe { *<*const _>::from(self).cast::<u32>() }
    }

    pub fn content<'a>(&self, input: &'a [u8]) -> Option<&'a str> {
        let result = match self {
            Event::Unparsed(content) => content.content(input),
            Event::LineFeed => return None,
            Event::Exit => return None,
            Event::Text(content) => content.content(input),
            Event::EnterParagraph => return None,
            Event::ThematicBreak => return None,
            Event::EnterCodeBlock => return None,
            Event::EnterCodeBlockMeta => return None,
            Event::EnterCodeBlockContent => return None,
        };

        Some(result)
    }
}
