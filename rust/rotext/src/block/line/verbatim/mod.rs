#[cfg(test)]
mod tests;

use std::ops::Range;

use crate::{
    block::{
        line::{global_phase, parse_common_end, ParseCommonEndOutput},
        types::CursorContext,
        utils::count_continuous_character,
    },
    events::{NewLine, VerbatimEscaping},
};

use super::CommonEnd;

#[derive(Clone)]
pub struct EndCondition {
    pub on_fence: Option<Fence>,
}
#[derive(Clone)]
pub struct Fence {
    pub character: u8,
    pub minimum_count: usize,
}

#[derive(Debug, PartialEq, Eq)]
pub enum End {
    Eof,
    NewLine(NewLine),
    VerbatimEscaping(VerbatimEscaping),
    Fence,
    None,
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
        Self::NewLine(value)
    }
}
impl From<VerbatimEscaping> for End {
    fn from(value: VerbatimEscaping) -> Self {
        Self::VerbatimEscaping(value)
    }
}
impl End {
    pub fn is_verbatim_escaping(&self) -> bool {
        matches!(self, End::VerbatimEscaping(_))
    }
}

pub struct AtLineBeginning {
    pub indent: usize,
}

pub fn parse<TCtx: CursorContext>(
    input: &[u8],
    ctx: &mut TCtx,
    end_condition: EndCondition,
    spaces_before: usize,
    mut at_line_beginning: Option<AtLineBeginning>,
) -> (Range<usize>, End) {
    let mut range = ctx.cursor()..(ctx.cursor());

    if let Some(AtLineBeginning { indent }) = at_line_beginning {
        if indent < spaces_before {
            range.start -= spaces_before - indent;
        }
    } else {
        range.start -= spaces_before;
    }

    loop {
        let char = input.get(ctx.cursor());
        let char = match parse_common_end(input, ctx, char) {
            ParseCommonEndOutput::Some(end) => {
                break (range, end.into());
            }
            ParseCommonEndOutput::NoneButMetSpace => {
                ctx.move_cursor_forward(" ".len());
                range.end = ctx.cursor();
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

        if let Some(cond) = &end_condition.on_fence {
            if at_line_beginning.is_some() && input.get(ctx.cursor()) == Some(&cond.character) {
                let count = 1 + count_continuous_character(input, cond.character, ctx.cursor() + 1);
                ctx.move_cursor_forward(count);
                if count < cond.minimum_count {
                    range.end = ctx.cursor();
                    at_line_beginning = None;
                    continue;
                }

                break (range, End::Fence);
            }
        }

        ctx.move_cursor_forward(1);
        range.end = ctx.cursor();
    }
}
