use std::ops::Range;

use crate::types::{BlockId, LineNumber};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum EventType {
    // 在块级阶段产出，由行内阶段消耗。
    Unparsed = 255,

    // 在块级阶段与行内阶段产出。
    Raw = 254, // TODO: 也许不应该有，而是为 NCR 专门创建一个（或两个）事件？
    NewLine = 201,
    VerbatimEscaping = 202,
    Text = 203,

    // 在块级阶段产出。
    ThematicBreak = 8,
    EnterParagraph = 7,
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
    EnterTable = 31,
    IndicateCodeBlockCode = 22,
    IndicateTableCaption = 35,
    IndicateTableRow = 32,
    IndicateTableHeaderCell = 33,
    IndicateTableDataCell = 34,
    ExitBlock = 99,

    // 在行内阶段产出。
    RefLink = 101,
    Dicexp = 102,
    EnterCodeSpan = 111,
    EnterStrong = 112,
    EnterStrikethrough = 113,
    EnterInternalLink = 121,
    ExitInline = 199,
}

#[cfg(test)]
impl From<u8> for EventType {
    fn from(value: u8) -> Self {
        unsafe { std::mem::transmute(value) }
    }
}

#[rotext_internal_macros::simple_sub_enum_for_event(
    current_mod_path = crate::events,
    enum_guard_macro_name = ev,
    debug_group_tester_macro_name = is_event_of,
    Block | InlineInput | Inline | Blend
)]
#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum Event {
    /// 留给下个阶段解析。
    #[groups(Block | InlineInput)]
    Unparsed(Range<usize>) = EventType::Unparsed as u8,

    /// 原封不动地渲染至输出。
    #[groups(Inline | Blend)]
    Raw(Range<usize>) = EventType::Raw as u8,

    /// 逐字转义。
    ///
    /// NOTE: 内容包含开头和结尾各可能存在的一个空格，省略上述空格的处理是在块级
    /// 阶段将 VerbatimEscaping 变换为 Text 时进行。
    #[groups(Block | InlineInput | Inline | Blend)]
    VerbatimEscaping(VerbatimEscaping) = EventType::VerbatimEscaping as u8,

    /// 换行，在全局阶段由 CR 与 LF 而来。
    #[groups(Block | InlineInput | Inline | Blend)]
    NewLine(NewLine) = EventType::NewLine as u8,

    /// 文本。
    #[groups(Block | Inline | Blend)]
    Text(Range<usize>) = EventType::Text as u8,

    /// 分割线。
    #[groups(Block | Blend)]
    ThematicBreak(ThematicBreak) = EventType::ThematicBreak as u8,

    /// 进入段落。
    #[groups(Block | Blend)]
    EnterParagraph(BlockWithId) = EventType::EnterParagraph as u8,
    /// 进入一级标题。
    #[groups(Block | Blend)]
    EnterHeading1(BlockWithId) = EventType::EnterHeading1 as u8,
    /// 进入二级标题。
    #[groups(Block | Blend)]
    EnterHeading2(BlockWithId) = EventType::EnterHeading2 as u8,
    /// 进入三级标题。
    #[groups(Block | Blend)]
    EnterHeading3(BlockWithId) = EventType::EnterHeading3 as u8,
    /// 进入四级标题。
    #[groups(Block | Blend)]
    EnterHeading4(BlockWithId) = EventType::EnterHeading4 as u8,
    /// 进入五级标题。
    #[groups(Block | Blend)]
    EnterHeading5(BlockWithId) = EventType::EnterHeading5 as u8,
    /// 进入六级标题。
    #[groups(Block | Blend)]
    EnterHeading6(BlockWithId) = EventType::EnterHeading6 as u8,
    /// 进入块引用
    #[groups(Block | Blend)]
    EnterBlockQuote(BlockWithId) = EventType::EnterBlockQuote as u8,
    /// 进入有序列表
    #[groups(Block | Blend)]
    EnterOrderedList(BlockWithId) = EventType::EnterOrderedList as u8,
    /// 进入无序列表
    #[groups(Block | Blend)]
    EnterUnorderedList(BlockWithId) = EventType::EnterUnorderedList as u8,
    /// 进入列表项
    #[groups(Block | Blend)]
    EnterListItem(BlockWithId) = EventType::EnterListItem as u8,
    /// 进入描述列表
    #[groups(Block | Blend)]
    EnterDescriptionList(BlockWithId) = EventType::EnterDescriptionList as u8,
    /// 进入描述术语
    #[groups(Block | Blend)]
    EnterDescriptionTerm(BlockWithId) = EventType::EnterDescriptionTerm as u8,
    /// 进入描述详情
    #[groups(Block | Blend)]
    EnterDescriptionDetails(BlockWithId) = EventType::EnterDescriptionDetails as u8,
    /// 进入代码块。
    #[groups(Block | Blend)]
    EnterCodeBlock(BlockWithId) = EventType::EnterCodeBlock as u8,
    /// 进入表格。
    #[groups(Block | Blend)]
    EnterTable(BlockWithId) = EventType::EnterTable as u8,

    /// 指示到达代码块的代码部分。
    #[groups(Block | Blend)]
    IndicateCodeBlockCode = EventType::IndicateCodeBlockCode as u8,
    /// 指示到达表格标题。
    #[groups(Block | Blend)]
    IndicateTableCaption = EventType::IndicateTableCaption as u8,
    /// 指示到达（新）表格行。
    #[groups(Block | Blend)]
    IndicateTableRow = EventType::IndicateTableRow as u8,
    /// 指示到达（新）表格头部单元格。
    #[groups(Block | Blend)]
    IndicateTableHeaderCell = EventType::IndicateTableHeaderCell as u8,
    /// 指示到达（新）表格数据单元格。
    #[groups(Block | Blend)]
    IndicateTableDataCell = EventType::IndicateTableDataCell as u8,

    /// 退出一层块级的 “进入…”。
    #[groups(Block | Blend)]
    ExitBlock(ExitBlock) = EventType::ExitBlock as u8,

    /// 引用链接。
    #[groups(Inline | Blend)]
    RefLink(Range<usize>) = EventType::RefLink as u8,
    /// Dicexp。
    #[groups(Inline | Blend)]
    Dicexp(Range<usize>) = EventType::Dicexp as u8,

    /// 进入行内代码。
    #[groups(Inline | Blend)]
    EnterCodeSpan = EventType::EnterCodeSpan as u8,
    /// 进入加粗强调。
    #[groups(Inline | Blend)]
    EnterStrong = EventType::EnterStrong as u8,
    /// 进入删除线。
    #[groups(Inline | Blend)]
    EnterStrikethrough = EventType::EnterStrikethrough as u8,

    // 进入内部链接。
    #[groups(Inline | Blend)]
    EnterInternalLink(Range<usize>) = EventType::EnterInternalLink as u8,

    /// 退出一层行内的 “进入…”。
    #[groups(Inline | Blend)]
    ExitInline = EventType::ExitInline as u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerbatimEscaping {
    pub content: Range<usize>,
    pub is_closed_forcedly: bool,
    pub line_after: LineNumber,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewLine {
    pub line_after: LineNumber,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockWithId {
    pub id: BlockId,
}
impl From<BlockId> for BlockWithId {
    fn from(value: BlockId) -> Self {
        Self { id: value }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThematicBreak {
    pub id: BlockId,
    pub line: LineNumber,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExitBlock {
    pub id: BlockId,
    pub start_line: LineNumber,
    pub end_line: LineNumber,
}

impl Event {
    #[cfg(test)]
    pub fn discriminant(&self) -> u8 {
        unsafe { *<*const _>::from(self).cast::<u8>() }
    }

    pub fn content<'a>(&self, input: &'a [u8]) -> Option<&'a str> {
        let result = match self {
            Event::Unparsed(content)
            | Event::Raw(content)
            | Event::VerbatimEscaping(VerbatimEscaping { content, .. })
            | Event::Text(content)
            | Event::RefLink(content)
            | Event::Dicexp(content)
            | Event::EnterInternalLink(content) => unsafe {
                std::str::from_utf8_unchecked(&input[content.clone()])
            },
            Event::NewLine(_)
            | Event::ThematicBreak(_)
            | Event::EnterParagraph(_)
            | Event::EnterHeading1(_)
            | Event::EnterHeading2(_)
            | Event::EnterHeading3(_)
            | Event::EnterHeading4(_)
            | Event::EnterHeading5(_)
            | Event::EnterHeading6(_)
            | Event::EnterBlockQuote(_)
            | Event::EnterOrderedList(_)
            | Event::EnterUnorderedList(_)
            | Event::EnterListItem(_)
            | Event::EnterDescriptionList(_)
            | Event::EnterDescriptionTerm(_)
            | Event::EnterDescriptionDetails(_)
            | Event::EnterCodeBlock(_)
            | Event::EnterTable(_)
            | Event::IndicateCodeBlockCode
            | Event::IndicateTableCaption
            | Event::IndicateTableRow
            | Event::IndicateTableHeaderCell
            | Event::IndicateTableDataCell
            | Event::ExitBlock(_)
            | Event::EnterCodeSpan
            | Event::EnterStrong
            | Event::EnterStrikethrough
            | Event::ExitInline => return None,
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
            Event::VerbatimEscaping(VerbatimEscaping { line_after, .. })
            | Event::NewLine(NewLine { line_after }) => {
                let flag_ln_after = format!(">ln:{}", line_after.value());
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

impl Event {
    pub fn is_block_event_that_opens_inline_phase(&self) -> bool {
        #[cfg(debug_assertions)]
        debug_assert!(is_event_of!(Block, self));

        matches!(
            self,
            ev!(Block, EnterParagraph(_))
                | ev!(Block, EnterHeading1(_))
                | ev!(Block, EnterHeading2(_))
                | ev!(Block, EnterHeading3(_))
                | ev!(Block, EnterHeading4(_))
                | ev!(Block, EnterHeading5(_))
                | ev!(Block, EnterHeading6(_))
        )
    }

    pub fn is_block_event_that_closes_inline_phase(&self) -> bool {
        #[cfg(debug_assertions)]
        debug_assert!(is_event_of!(Block, self));

        matches!(self, ev!(Block, ExitBlock(_)))
    }
}
