use crate::{block::types::CursorContext, types::LineNumber};

#[derive(Debug, PartialEq, Eq)]
pub struct MockCursorContext {
    pub cursor: usize,
    pub current_line: LineNumber,
}
impl CursorContext for MockCursorContext {
    fn cursor(&self) -> usize {
        self.cursor
    }

    fn set_cursor(&mut self, cursor: usize) {
        self.cursor = cursor;
    }

    fn move_cursor_forward(&mut self, n: usize) {
        self.cursor += n;
    }

    fn current_line(&self) -> LineNumber {
        self.current_line
    }

    fn increase_current_line(&mut self, _is_significant: bool) {
        self.current_line.increase()
    }
}
