#[cfg(test)]
mod tests;

use crate::{
    block::{
        types::{CursorContext, YieldContext},
        utils::count_continuous_character,
    },
    common::m,
    events::VerbatimEscaping,
    types::Tym,
    BlockEvent,
};

#[derive(Debug, PartialEq, Eq)]
pub enum Output {
    VerbatimEscaping(VerbatimEscaping),
    /// 没有输出。（如在解析出注释时。）
    None,
}
impl From<VerbatimEscaping> for Output {
    fn from(value: VerbatimEscaping) -> Self {
        Self::VerbatimEscaping(value)
    }
}

pub fn parse<TCtx: CursorContext>(input: &[u8], ctx: &mut TCtx, first_char: u8) -> Option<Output> {
    debug_assert!(matches!(input.get(ctx.cursor()), Some(&char) if char == first_char));

    if first_char != m!('<') {
        return None;
    }

    match input.get(ctx.cursor() + 1)? {
        m!('`') => {
            ctx.move_cursor_forward("<`".len());
            let verbatim_escaping = parse_verbatim_escaping(input, ctx);
            Some(Output::VerbatimEscaping(verbatim_escaping))
        }
        m!('%') => {
            ctx.move_cursor_forward("<%".len());
            parse_comment(input, ctx);
            Some(Output::None)
        }
        _ => None,
    }
}

fn parse_verbatim_escaping<TCtx: CursorContext>(input: &[u8], ctx: &mut TCtx) -> VerbatimEscaping {
    let count = count_continuous_character(input, m!('`'), ctx.cursor());
    ctx.move_cursor_forward(count);
    let backticks = "`".len() + count;

    let mut start = ctx.cursor();
    let has_leading_space = if let Some(&char) = input.get(start) {
        ctx.move_cursor_forward(1);
        char == b' '
    } else {
        false
    };

    let mut continuous_backticks = 0;
    while let Some(char) = input.get(ctx.cursor()).copied() {
        match char {
            m!('`') => continuous_backticks += 1,
            m!('>') if continuous_backticks == backticks => {
                let mut end = ctx.cursor() - continuous_backticks;
                ctx.move_cursor_forward(">".len());
                if end - start >= 2 {
                    if has_leading_space {
                        start += 1;
                    }
                    // SAFETY: `end` < `ctx.cursor()` < `input.len()`.
                    if unsafe { *input.get_unchecked(end - 1) } == b' ' {
                        end -= 1;
                    }
                }

                return VerbatimEscaping {
                    content: start..end,
                    is_closed_forcedly: false,
                    line_after: ctx.current_line(),
                };
            }
            b'\r' | b'\n' => {
                ctx.increase_current_line();
                if char == b'\r' && matches!(input.get(ctx.cursor() + 1), Some(b'\n')) {
                    ctx.move_cursor_forward(1);
                }
                continuous_backticks = 0
            }
            _ => continuous_backticks = 0,
        }

        ctx.move_cursor_forward(1);
    }

    if has_leading_space && start < input.len() {
        start += 1;
    }

    VerbatimEscaping {
        content: start..input.len(),
        is_closed_forcedly: true,
        line_after: ctx.current_line(),
    }
}

fn parse_comment<TCtx: CursorContext>(input: &[u8], ctx: &mut TCtx) {
    let mut depth = 1;

    while depth > 0 {
        let Some(char) = input.get(ctx.cursor()).copied() else {
            break;
        };
        match char {
            m!('<') => match input.get(ctx.cursor() + 1) {
                Some(m!('%')) => {
                    ctx.move_cursor_forward("<%".len());
                    depth += 1;
                }
                Some(m!('`')) => {
                    ctx.move_cursor_forward("<`".len());
                    parse_verbatim_escaping(input, ctx);
                }
                _ => ctx.move_cursor_forward(1),
            },
            m!('%') if input.get(ctx.cursor() + 1) == Some(&b'>') => {
                ctx.move_cursor_forward("%>".len());
                depth -= 1;
            }
            b'\r' | b'\n' => {
                ctx.increase_current_line();
                ctx.move_cursor_forward(1);
                if char == b'\r' && matches!(input.get(ctx.cursor() + 1), Some(b'\n')) {
                    ctx.move_cursor_forward(1);
                }
            }
            _ => ctx.move_cursor_forward(1),
        }
    }
}

pub fn process_verbatim_escaping<TCtx: YieldContext>(
    ctx: &mut TCtx,
    verbatim_escaping: VerbatimEscaping,
) -> Tym<1> {
    ctx.r#yield(BlockEvent::VerbatimEscaping(verbatim_escaping))
}
