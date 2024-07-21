use subenum::subenum;

use crate::common::Range;

#[derive(Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum EventType {
    // 在全局阶段产出，由块级阶段和行内阶段逐渐消耗。
    Unparsed = 1,

    // 在全局阶段产出，在块级阶段消耗（转化为 [EventType::Text）。
    VerbatimEscaping = 101,

    // 换行相关，皆在全局阶段产出。其中 CR 在块级阶段消耗（可能转化为 LF），LF
    // 在块级阶段、行内阶段保留或消耗。
    CarriageReturn = 201,
    LineFeed = 202,

    // 在块级阶段产出，由 CR 与 LF 而来。
    LineBreak = 299,

    // 在块级阶段与行内阶段产出。
    Text = 1001,
    Exit = 1011,
    Separator = 1021,

    // 在块级阶段产出。
    EnterParagraph = 10001,
    ThematicBreak = 10011,
    EnterHeading1 = 10021,
    EnterHeading2 = 10022,
    EnterHeading3 = 10023,
    EnterHeading4 = 10024,
    EnterHeading5 = 10025,
    EnterHeading6 = 10026,
    EnterBlockQuote = 10031,
    /// XXX: 数字是临时的。
    EnterCodeBlock = 19011,
}

#[cfg(test)]
impl From<u32> for EventType {
    fn from(value: u32) -> Self {
        unsafe { std::mem::transmute(value) }
    }
}

#[subenum(
    GlobalEvent,
    BlockEvent,
    InlineLevelParseInputEvent,
    InlineEvent,
    BlendEvent
)]
#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(u32)]
pub enum Event {
    /// 留给下个阶段解析。
    #[subenum(GlobalEvent, BlockEvent, InlineLevelParseInputEvent)]
    Unparsed(Range) = EventType::Unparsed as u32,

    /// 逐字文本转义。
    ///
    /// NOTE: 内容包含开头和结尾各可能存在的一个空格，省略上述空格的处理是在块级
    /// 阶段将 VerbatimEscaping 变换为 Text 时进行。
    #[subenum(GlobalEvent)]
    VerbatimEscaping {
        content: Range,
        is_closed_forcedly: bool,
    } = EventType::VerbatimEscaping as u32,

    #[subenum(GlobalEvent)]
    CarriageReturn { index: usize } = EventType::CarriageReturn as u32,
    /// LF 换行。对于块级阶段，只在解析内容时产生。
    #[subenum(GlobalEvent)]
    LineFeed { index: usize } = EventType::LineFeed as u32,

    /// 换行，由 CR 与 LF 而来。
    #[subenum(BlockEvent, InlineLevelParseInputEvent, InlineEvent, BlendEvent)]
    LineBreak = EventType::LineBreak as u32,

    /// 文本。
    #[subenum(BlockEvent, InlineLevelParseInputEvent, InlineEvent, BlendEvent)]
    Text(Range) = EventType::Text as u32,
    /// 退出一层 “进入…”。
    #[subenum(BlockEvent, InlineEvent, BlendEvent)]
    Exit = EventType::Exit as u32,
    /// 分隔符。出现于块级嵌入包含、块级扩展与代码块的内容中。其中代码块会用它来
    /// 隔开 info string 与实际代码内容。
    #[subenum(BlockEvent, InlineEvent, BlendEvent)]
    Separator = EventType::Separator as u32,

    /// 进入段落。
    #[subenum(BlockEvent, BlendEvent)]
    EnterParagraph = EventType::EnterParagraph as u32,
    /// 分割线。
    #[subenum(BlockEvent, BlendEvent)]
    ThematicBreak = EventType::ThematicBreak as u32,
    /// 一级标题。
    #[subenum(BlockEvent, BlendEvent)]
    EnterHeading1 = EventType::EnterHeading1 as u32,
    /// 二级标题。
    #[subenum(BlockEvent, BlendEvent)]
    EnterHeading2 = EventType::EnterHeading2 as u32,
    /// 三级标题。
    #[subenum(BlockEvent, BlendEvent)]
    EnterHeading3 = EventType::EnterHeading3 as u32,
    /// 四级标题。
    #[subenum(BlockEvent, BlendEvent)]
    EnterHeading4 = EventType::EnterHeading4 as u32,
    /// 五级标题。
    #[subenum(BlockEvent, BlendEvent)]
    EnterHeading5 = EventType::EnterHeading5 as u32,
    /// 六级标题。
    #[subenum(BlockEvent, BlendEvent)]
    EnterHeading6 = EventType::EnterHeading6 as u32,
    /// 块引用
    #[subenum(BlockEvent, BlendEvent)]
    EnterBlockQuote = EventType::EnterBlockQuote as u32,
    /// 代码块。
    #[subenum(BlockEvent, BlendEvent)]
    EnterCodeBlock = EventType::EnterCodeBlock as u32,
}

impl Event {
    #[cfg(test)]
    pub fn discriminant(&self) -> u32 {
        unsafe { *<*const _>::from(self).cast::<u32>() }
    }

    pub fn content<'a>(&self, input: &'a [u8]) -> Option<&'a str> {
        let result = match self {
            Event::Unparsed(content) => content.content(input),
            Event::VerbatimEscaping { content, .. } => content.content(input),
            Event::CarriageReturn { .. } => return None,
            Event::LineFeed { .. } => return None,
            Event::LineBreak => return None,
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
            Event::EnterBlockQuote => return None,
            Event::EnterCodeBlock => return None,
        };

        Some(result)
    }

    #[cfg(test)]
    pub fn assertion_flags(&self) -> Option<std::collections::HashSet<&'static str>> {
        let mut flags = std::collections::HashSet::new();

        match *self {
            Event::VerbatimEscaping {
                content: _,
                is_closed_forcedly,
            } if is_closed_forcedly => {
                flags.insert("F");
            }
            _ => {}
        }

        if !flags.is_empty() {
            Some(flags)
        } else {
            None
        }
    }
}
