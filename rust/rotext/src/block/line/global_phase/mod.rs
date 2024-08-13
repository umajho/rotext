#[cfg(test)]
mod tests;

use crate::{
    block::{
        types::{CursorContext, Tym, YieldContext},
        utils::count_continuous_character,
    },
    common::m,
    events::VerbatimEscaping,
    BlockEvent,
};

#[derive(Debug, PartialEq, Eq)]
pub enum Output {
    VerbatimEscaping(VerbatimEscaping),
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
        _ => None,
    }
}

pub fn parse_verbatim_escaping<TCtx: CursorContext>(
    input: &[u8],
    ctx: &mut TCtx,
) -> VerbatimEscaping {
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
    while ctx.cursor() < input.len() {
        // SAFETY: `ctx.cursor()` < `input.len()`.
        let char = unsafe { *input.get_unchecked(ctx.cursor()) };
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

pub fn process_verbatim_escaping<TCtx: YieldContext>(
    ctx: &mut TCtx,
    verbatim_escaping: VerbatimEscaping,
) -> Tym<1> {
    ctx.r#yield(BlockEvent::VerbatimEscaping(verbatim_escaping))
}
