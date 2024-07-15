#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Range {
    start: usize,
    length: usize,
}

impl Range {
    #[inline(always)]
    pub fn new(start: usize, length: usize) -> Range {
        Range { start, length }
    }

    #[inline(always)]
    pub fn start(&self) -> usize {
        self.start
    }

    #[inline(always)]
    pub fn length(&self) -> usize {
        self.length
    }

    #[inline(always)]
    pub fn set_length(&mut self, value: usize) {
        self.length = value;
    }

    pub fn content(&self, input: &[u8]) -> String {
        let slice = &input[self.start..self.start + self.length];
        String::from_utf8(slice.to_vec()).unwrap()
    }
}
