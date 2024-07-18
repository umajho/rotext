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
    /// 分隔符。出现于块级嵌入包含、块级扩展与代码块的内容中。其中代码块会用它来
    /// 隔开 info string 与实际代码内容。
    Separator = EventType::Separator as u32,

    /// 进入段落。
    EnterParagraph = EventType::EnterParagraph as u32,
    /// 分割线
    ThematicBreak = EventType::ThematicBreak as u32,
    /// 一级标题
    EnterHeading1 = EventType::EnterHeading1 as u32,
    /// 二级标题
    EnterHeading2 = EventType::EnterHeading2 as u32,
    /// 三级标题
    EnterHeading3 = EventType::EnterHeading3 as u32,
    /// 四级标题
    EnterHeading4 = EventType::EnterHeading4 as u32,
    /// 五级标题
    EnterHeading5 = EventType::EnterHeading5 as u32,
    /// 六级标题
    EnterHeading6 = EventType::EnterHeading6 as u32,
    /// 代码块。
    EnterCodeBlock = EventType::EnterCodeBlock as u32,

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
            Event::Separator => return None,
            Event::Text(content) => content.content(input),
            Event::EnterParagraph => return None,
            Event::ThematicBreak => return None,
            Event::EnterHeading1 => return None,
            Event::EnterHeading2 => return None,
            Event::EnterHeading3 => return None,
            Event::EnterHeading4 => return None,
            Event::EnterHeading5 => return None,
            Event::EnterHeading6 => return None,
            Event::EnterCodeBlock => return None,
        };

        Some(result)
    }
}
