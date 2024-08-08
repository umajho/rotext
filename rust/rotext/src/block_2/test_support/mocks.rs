use crate::{block_2::types::CursorContext, types::LineNumber};

#[derive(Debug, PartialEq, Eq)]
pub struct MockCursorContext {
    pub cursor: usize,
    pub current_line: LineNumber,
}
impl CursorContext for MockCursorContext {
    fn cursor(&self) -> usize {
        self.cursor
    }

    fn move_cursor_forward(&mut self, n: usize) {
        self.cursor += n;
    }

    fn current_line(&self) -> LineNumber {
        self.current_line
    }

    fn increase_current_line(&mut self) {
        self.current_line.increase()
    }
}
