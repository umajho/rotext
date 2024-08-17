use crate::inline::types::CursorContext;

#[derive(Debug, PartialEq, Eq)]
pub struct MockCursorContext {
    pub cursor: usize,
}
impl CursorContext for MockCursorContext {
    fn cursor(&self) -> usize {
        self.cursor
    }

    fn set_cursor(&mut self, value: usize) {
        self.cursor = value;
    }

    fn move_cursor_forward(&mut self, n: usize) {
        self.cursor += n;
    }
}
