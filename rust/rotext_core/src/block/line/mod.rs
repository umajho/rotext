use crate::{events::NewLine, internal_utils::string::is_whitespace};

use super::{types::CursorContext, utils::move_cursor_over_line_break};

pub mod global_phase;
pub mod normal;
pub mod verbatim;

enum ParseCommonEndOutput {
    Some(CommonEnd),
    NoneButMetSpace,
    None(u8),
}
enum CommonEnd {
    Eof,
    NewLine(NewLine),
}
impl From<NewLine> for CommonEnd {
    fn from(value: NewLine) -> Self {
        Self::NewLine(value)
    }
}

fn parse_common_end<TCtx: CursorContext>(
    input: &[u8],
    ctx: &mut TCtx,
    char: Option<&u8>,
) -> ParseCommonEndOutput {
    let Some(&char) = char else {
        return ParseCommonEndOutput::Some(CommonEnd::Eof);
    };

    match char {
        b'\r' | b'\n' => {
            ctx.increase_current_line(true);
            move_cursor_over_line_break(ctx, input);
            ParseCommonEndOutput::Some(
                NewLine {
                    line_after: ctx.current_line(),
                }
                .into(),
            )
        }
        char if is_whitespace!(char) => ParseCommonEndOutput::NoneButMetSpace,
        _ => ParseCommonEndOutput::None(char),
    }
}
