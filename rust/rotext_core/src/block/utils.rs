use crate::block::types::CursorContext;

/// 调用本函数时应已经确认 `ctx.cursor()` 为 `\r` 或 `\n`。
pub fn move_cursor_over_line_break<C: CursorContext>(ctx: &mut C, input: &[u8]) {
    ctx.move_cursor_forward(1);
    if input.get(ctx.cursor()) == Some(&b'\n') && input.get(ctx.cursor() - 1) == Some(&b'\r') {
        ctx.move_cursor_forward(1);
    }
}

#[cfg(feature = "block-id")]
use crate::types::BlockId;

#[cfg(feature = "block-id")]
pub struct BlockIdGenerator(usize);
#[cfg(feature = "block-id")]
impl BlockIdGenerator {
    pub fn new() -> Self {
        Self(0)
    }

    #[cfg(feature = "block-id")]
    pub fn pop(&mut self) -> BlockId {
        self.0 += 1;
        BlockId::new(self.0)
    }
}
