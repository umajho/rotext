use crate::types::LineNumber;

/// 只提供读取与移动 cursor，以及读取及增加当前行数的上下文。
pub trait CursorContext {
    fn cursor(&self) -> usize;
    fn move_cursor_forward(&mut self, n: usize);

    fn current_line(&self) -> LineNumber;
    fn increase_current_line(&mut self);
}
