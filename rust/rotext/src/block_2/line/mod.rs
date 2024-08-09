use crate::events::NewLine;

use super::types::CursorContext;

pub mod normal;
pub mod verbatim;

mod global_phase;

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
            ctx.increase_current_line();
            ctx.move_cursor_forward(1);
            if char == b'\r' && input.get(ctx.cursor()) == Some(&b'\n') {
                ctx.move_cursor_forward(1);
            }
            ParseCommonEndOutput::Some(
                NewLine {
                    line_after: ctx.current_line(),
                }
                .into(),
            )
        }
        b' ' => ParseCommonEndOutput::NoneButMetSpace,
        _ => ParseCommonEndOutput::None(char),
    }
}
