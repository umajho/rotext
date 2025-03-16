#[cfg(test)]
mod tests;

use core::ops::Range;

use crate::{
    block::{branch::braced::table, types::CursorContext},
    common::m,
    events::{NewLine, VerbatimEscaping},
    internal_utils::string::{count_continuous_character_with_maximum, is_whitespace},
};

use super::{global_phase, parse_common_end, CommonEnd, ParseCommonEndOutput};

#[derive(Debug, Clone)]
pub struct EndCondition {
    pub on_atx_closing: Option<AtxClosing>,
    pub on_table_related: Option<TableRelated>,
    pub on_description_definition_opening: bool,
}
/// 类似于 CommonMark 中 ATX 风格的 Headings 中的闭合部分，位于空格之后，外部结构的结
/// 尾之前（除了常规的换行和文档结束，“结尾” 还可能是 [braced] 的闭合部分），标记都
/// 是相同的字符。
///
/// 与 CommonMark 不同的是，此部分标记的长度必须与对应的开启部分的相同。
#[derive(Debug, Clone)]
pub struct AtxClosing {
    pub character: u8,
    pub count: usize,
}
#[derive(Debug, Clone)]
pub struct TableRelated {
    pub is_caption_applicable: bool,
}

/// [parse] 结束解析的原因。
///
/// 由于 ATX 闭合总是会被其他变体所替代，因此其中不存在对应于 ATX 闭合的变体。
#[derive(Debug, PartialEq, Eq)]
pub enum End {
    Eof,
    /// 实际上在处理之前总是 Some，而在处理之后就 drop 掉了。
    NewLine(Option<NewLine>),
    VerbatimEscaping(VerbatimEscaping),
    TableRelated(table::TableRelatedEnd),
    DoublePipes,
    DescriptionDefinitionOpening,
    None,
}
impl From<table::TableRelatedEnd> for End {
    fn from(value: table::TableRelatedEnd) -> Self {
        Self::TableRelated(value)
    }
}
impl From<CommonEnd> for End {
    fn from(value: CommonEnd) -> Self {
        match value {
            CommonEnd::Eof => End::Eof,
            CommonEnd::NewLine(new_line) => new_line.into(),
        }
    }
}
impl From<NewLine> for End {
    fn from(value: NewLine) -> Self {
        Self::NewLine(Some(value))
    }
}
impl From<VerbatimEscaping> for End {
    fn from(value: VerbatimEscaping) -> Self {
        Self::VerbatimEscaping(value)
    }
}
impl End {
    pub fn try_take_new_line(&mut self) -> Option<NewLine> {
        match self {
            End::NewLine(new_line) => new_line.take(),
            _ => None,
        }
    }

    pub fn is_verbatim_escaping(&self) -> bool {
        matches!(self, End::VerbatimEscaping(_))
    }
}

pub enum ContentBefore {
    NotSpace(usize),
    Space,
}

/// 解析一般的一行中的一段内容，即并非逐字内容的一行。
///
/// 有关参数 `content_before` 的说明见 [leaf::paragraph::enter_if_not_blank]。
///
/// 调用者应确保调用时 cursor 所指的字符并非空格。拒绝开头有空格的情况是为了明确开头的这些空格
/// 应该由外部决定是保留还是省略，本函数不对此做处理。
pub fn parse<TCtx: CursorContext>(
    input: &[u8],
    ctx: &mut TCtx,
    end_condition: EndCondition,
    content_before: ContentBefore,
) -> (Range<usize>, End) {
    debug_assert!(input.get(ctx.cursor()).is_none_or(|c| !is_whitespace!(c)));

    let mut range = ctx.cursor()..(ctx.cursor());
    if let ContentBefore::NotSpace(n) = content_before {
        range.end += n;
    }
    let mut spaces = 0;

    let (mut range, end) = loop {
        let char = input.get(ctx.cursor());
        let char = match parse_common_end(input, ctx, char) {
            ParseCommonEndOutput::Some(end) => {
                break (range, end.into());
            }
            ParseCommonEndOutput::NoneButMetSpace => {
                ctx.move_cursor_forward(" ".len());
                spaces += " ".len();
                continue;
            }
            ParseCommonEndOutput::None(char) => char,
        };

        if let Some(output) = global_phase::parse(input, ctx, char) {
            match output {
                global_phase::Output::VerbatimEscaping(verbatim_escaping) => {
                    break (range, verbatim_escaping.into());
                }
                global_phase::Output::None => break (range, End::None),
            }
        }

        let is_after_space =
            spaces > 0 || (range.is_empty() && matches!(content_before, ContentBefore::Space));
        if let Some(cond) = &end_condition.on_atx_closing {
            if is_after_space && char == cond.character {
                let count = 1 + count_continuous_character_with_maximum(
                    input,
                    cond.character,
                    ctx.cursor() + 1,
                    cond.count - 1,
                );
                ctx.move_cursor_forward(count);
                if count != cond.count {
                    range.end = ctx.cursor();
                    spaces = 0;
                    continue;
                }

                let opts = ParseEndWhenConfirmedNoAtxClosingOptions {
                    is_range_empty: range.is_empty(),
                };
                let output =
                    parse_following_end_that_can_close_heading(input, ctx, &end_condition, opts);
                if let Some(following_end) = output.end {
                    // 如果有紧随 ATX 闭合的 [End]，后者可以代替前者，因为后者无论是哪种
                    // 都会退出 Heading。
                    break (range, following_end);
                } else {
                    // 如果潜在的 ATX 闭合之后没有紧随的其他 [End]，代表这其实不是 ATX
                    // 闭合，视为一般文本处理。
                    spaces = output.spaces_before_end;
                    range.end = ctx.cursor() - spaces;
                    continue;
                }
            }
        }

        {
            let opts = ParseEndWhenConfirmedNoAtxClosingOptions {
                is_range_empty: range.is_empty(),
            };
            if let Some(end) =
                parse_braced_element_related_end(input, ctx, &end_condition, char, opts)
            {
                break (range, end);
            }
        }

        if end_condition.on_description_definition_opening
            && is_after_space
            && char == m!(':')
            && input.get(ctx.cursor() + 1) == Some(&m!(':'))
        {
            let next_next_char = input.get(ctx.cursor() + 2);
            let result = match next_next_char {
                None | Some(b'\r' | b'\n') => Some(2),
                Some(char) if is_whitespace!(char) => Some(3),
                _ => None,
            };
            if let Some(to_move) = result {
                ctx.move_cursor_forward(to_move);
                break (range, End::DescriptionDefinitionOpening);
            }
        }

        ctx.move_cursor_forward(1);
        range.end = ctx.cursor();
        spaces = 0;
    };

    match end {
        End::VerbatimEscaping(_) | End::None => range.end += spaces,
        End::Eof
        | End::NewLine(_)
        | End::TableRelated(_)
        | End::DoublePipes
        | End::DescriptionDefinitionOpening => {}
    }

    (range, end)
}

struct ParseFollwingEndThatCanCloseHeadingOutput {
    spaces_before_end: usize,
    end: Option<End>,
}
/// 解析紧随于 ATX 闭合之后的、可以闭合 Heading 的 [End]。
///
/// “紧随” 是指中间最多只能有空格相隔。
///
/// 可以闭合当前 Heading 的 [End] 有 [CommonEnd] 与 [table::TableRelatedEnd]。
fn parse_following_end_that_can_close_heading<TCtx: CursorContext>(
    input: &[u8],
    ctx: &mut TCtx,
    end_condition: &EndCondition,
    opts: ParseEndWhenConfirmedNoAtxClosingOptions,
) -> ParseFollwingEndThatCanCloseHeadingOutput {
    let mut spaces_before_end = 0;
    loop {
        let char = input.get(ctx.cursor());
        let char = match parse_common_end(input, ctx, char) {
            ParseCommonEndOutput::Some(end) => {
                return ParseFollwingEndThatCanCloseHeadingOutput {
                    spaces_before_end,
                    end: Some(end.into()),
                };
            }
            ParseCommonEndOutput::NoneButMetSpace => {
                ctx.move_cursor_forward(" ".len());
                spaces_before_end += 1;
                continue;
            }
            ParseCommonEndOutput::None(char) => char,
        };

        if let Some(end) = parse_braced_element_related_end(input, ctx, end_condition, char, opts) {
            return ParseFollwingEndThatCanCloseHeadingOutput {
                spaces_before_end,
                end: Some(end),
            };
        }

        return ParseFollwingEndThatCanCloseHeadingOutput {
            spaces_before_end,
            end: None,
        };
    }
}

struct ParseEndWhenConfirmedNoAtxClosingOptions {
    is_range_empty: bool,
}
/// 确定了没有 ATX Heading 闭合部分之后进行的接下来的解析。
///
/// 目前只可能返回 [End::TableRelated] 及 [End::DoublePipes]。
fn parse_braced_element_related_end<TCtx: CursorContext>(
    input: &[u8],
    ctx: &mut TCtx,
    end_condition: &EndCondition,
    first_char: u8,
    opts: ParseEndWhenConfirmedNoAtxClosingOptions,
) -> Option<End> {
    if let Some(end_condition) = &end_condition.on_table_related {
        let is_caption_applicable = opts.is_range_empty && end_condition.is_caption_applicable;
        let end =
            table::parse_end(input, ctx, first_char, is_caption_applicable).map(End::TableRelated);
        if end.is_some() {
            return end;
        }
    }

    if end_condition.on_table_related.is_some()
        && first_char == m!('|')
        && input.get(ctx.cursor() + 1) == Some(&m!('|'))
    {
        ctx.move_cursor_forward("||".len());
        return Some(End::DoublePipes);
    }

    None
}
