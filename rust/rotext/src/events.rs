use subenum::subenum;

use crate::common::Range;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum EventType {
    // 在全局阶段产出，由块级阶段和行内阶段逐渐消耗。
    Unparsed = 255,

    // 在全局阶段产出，由 CR 与 LF 而来。
    NewLine = 201,
    VerbatimEscaping = 202,

    // 在块级阶段与行内阶段产出。
    Text = 203,
    Separator = 252,

    // 在块级阶段产出。
    EnterParagraph = 7,
    ThematicBreak = 8,
    EnterHeading1 = 1,
    EnterHeading2 = 2,
    EnterHeading3 = 3,
    EnterHeading4 = 4,
    EnterHeading5 = 5,
    EnterHeading6 = 6,
    EnterBlockQuote = 11,
    EnterOrderedList = 12,
    EnterUnorderedList = 13,
    EnterListItem = 14,
    EnterDescriptionList = 15,
    EnterDescriptionTerm = 16,
    EnterDescriptionDetails = 17,
    EnterCodeBlock = 21,
    ExitBlock = 99,
}

#[cfg(test)]
impl From<u8> for EventType {
    fn from(value: u8) -> Self {
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
#[repr(u8)]
pub enum Event {
    /// 留给下个阶段解析。
    #[subenum(GlobalEvent, BlockEvent, InlineLevelParseInputEvent)]
    Unparsed(Range) = EventType::Unparsed as u8,

    /// 逐字文本转义。
    ///
    /// NOTE: 内容包含开头和结尾各可能存在的一个空格，省略上述空格的处理是在块级
    /// 阶段将 VerbatimEscaping 变换为 Text 时进行。
    #[subenum(
        GlobalEvent,
        BlockEvent,
        InlineLevelParseInputEvent,
        InlineEvent,
        BlendEvent
    )]
    VerbatimEscaping(VerbatimEscaping) = EventType::VerbatimEscaping as u8,

    /// 换行，在全局阶段由 CR 与 LF 而来。
    #[subenum(
        GlobalEvent,
        BlockEvent,
        InlineLevelParseInputEvent,
        InlineEvent,
        BlendEvent
    )]
    NewLine(NewLine) = EventType::NewLine as u8,

    /// 文本。
    #[subenum(BlockEvent, InlineLevelParseInputEvent, InlineEvent, BlendEvent)]
    Text(Range) = EventType::Text as u8,
    /// 分隔符。出现于块级嵌入包含、块级扩展与代码块的内容中。其中代码块会用它来
    /// 隔开 info string 与实际代码内容。
    #[subenum(BlockEvent, InlineEvent, BlendEvent)]
    Separator = EventType::Separator as u8,

    /// 进入段落。
    #[subenum(BlockEvent, BlendEvent)]
    EnterParagraph = EventType::EnterParagraph as u8,
    /// 分割线。
    #[subenum(BlockEvent, BlendEvent)]
    ThematicBreak(ThematicBreak) = EventType::ThematicBreak as u8,
    /// 进入一级标题。
    #[subenum(BlockEvent, BlendEvent)]
    EnterHeading1 = EventType::EnterHeading1 as u8,
    /// 进入二级标题。
    #[subenum(BlockEvent, BlendEvent)]
    EnterHeading2 = EventType::EnterHeading2 as u8,
    /// 进入三级标题。
    #[subenum(BlockEvent, BlendEvent)]
    EnterHeading3 = EventType::EnterHeading3 as u8,
    /// 进入四级标题。
    #[subenum(BlockEvent, BlendEvent)]
    EnterHeading4 = EventType::EnterHeading4 as u8,
    /// 进入五级标题。
    #[subenum(BlockEvent, BlendEvent)]
    EnterHeading5 = EventType::EnterHeading5 as u8,
    /// 进入六级标题。
    #[subenum(BlockEvent, BlendEvent)]
    EnterHeading6 = EventType::EnterHeading6 as u8,
    /// 进入块引用
    #[subenum(BlockEvent, BlendEvent)]
    EnterBlockQuote = EventType::EnterBlockQuote as u8,
    /// 进入有序列表
    #[subenum(BlockEvent, BlendEvent)]
    EnterOrderedList = EventType::EnterOrderedList as u8,
    /// 进入无序列表
    #[subenum(BlockEvent, BlendEvent)]
    EnterUnorderedList = EventType::EnterUnorderedList as u8,
    /// 进入列表项
    #[subenum(BlockEvent, BlendEvent)]
    EnterListItem = EventType::EnterListItem as u8,
    /// 进入描述列表
    #[subenum(BlockEvent, BlendEvent)]
    EnterDescriptionList = EventType::EnterDescriptionList as u8,
    /// 进入描述术语
    #[subenum(BlockEvent, BlendEvent)]
    EnterDescriptionTerm = EventType::EnterDescriptionTerm as u8,
    /// 进入描述详情
    #[subenum(BlockEvent, BlendEvent)]
    EnterDescriptionDetails = EventType::EnterDescriptionDetails as u8,
    /// 进入代码块。
    #[subenum(BlockEvent, BlendEvent)]
    EnterCodeBlock = EventType::EnterCodeBlock as u8,
    /// 退出一层块级的 “进入…”。
    #[subenum(BlockEvent, InlineEvent, BlendEvent)]
    ExitBlock(ExitBlock) = EventType::ExitBlock as u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerbatimEscaping {
    pub content: Range,
    pub is_closed_forcedly: bool,
    #[cfg(feature = "line-number")]
    pub line_number_after: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewLine {
    #[cfg(feature = "line-number")]
    pub line_number_after: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThematicBreak {
    #[cfg(feature = "line-number")]
    pub line_number: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExitBlock {
    #[cfg(feature = "line-number")]
    pub start_line_number: usize,
    #[cfg(feature = "line-number")]
    pub end_line_number: usize,
}

impl Event {
    #[cfg(test)]
    pub fn discriminant(&self) -> u8 {
        unsafe { *<*const _>::from(self).cast::<u8>() }
    }

    pub fn content<'a>(&self, input: &'a [u8]) -> Option<&'a str> {
        let result = match self {
            Event::Unparsed(content)
            | Event::VerbatimEscaping(VerbatimEscaping { content, .. })
            | Event::Text(content) => content.content(input),
            Event::NewLine(_)
            | Event::ExitBlock(_)
            | Event::Separator
            | Event::EnterParagraph
            | Event::ThematicBreak(_)
            | Event::EnterHeading1
            | Event::EnterHeading2
            | Event::EnterHeading3
            | Event::EnterHeading4
            | Event::EnterHeading5
            | Event::EnterHeading6
            | Event::EnterBlockQuote
            | Event::EnterOrderedList
            | Event::EnterUnorderedList
            | Event::EnterListItem
            | Event::EnterDescriptionList
            | Event::EnterDescriptionTerm
            | Event::EnterDescriptionDetails
            | Event::EnterCodeBlock => return None,
        };

        Some(result)
    }

    #[cfg(test)]
    pub fn assertion_flags(&self) -> Option<std::collections::HashSet<&'static str>> {
        let mut flags = std::collections::HashSet::new();

        match self {
            Event::VerbatimEscaping(VerbatimEscaping {
                is_closed_forcedly, ..
            }) if *is_closed_forcedly => {
                flags.insert("F");
            }
            _ => {}
        }

        #[cfg(feature = "line-number")]
        match self {
            Event::VerbatimEscaping(VerbatimEscaping {
                line_number_after, ..
            })
            | Event::NewLine(NewLine { line_number_after }) => {
                let flag_ln_after = format!(">ln:{}", line_number_after);
                // 反正也只在测试时使用，为图开发方便，干脆就 leak 了。
                flags.insert(flag_ln_after.leak());
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

impl BlockEvent {
    pub fn opens_inline_phase(&self) -> bool {
        matches!(
            self,
            BlockEvent::EnterParagraph
                | BlockEvent::EnterHeading1
                | BlockEvent::EnterHeading2
                | BlockEvent::EnterHeading3
                | BlockEvent::EnterHeading4
                | BlockEvent::EnterHeading5
                | BlockEvent::EnterHeading6
                | BlockEvent::Separator
        )
    }

    pub fn closes_inline_phase(&self) -> bool {
        matches!(self, BlockEvent::ExitBlock(_) | BlockEvent::Separator)
    }
}
