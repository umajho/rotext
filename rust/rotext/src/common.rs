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

    #[inline(always)]
    pub fn increase_length(&mut self, delta: usize) {
        self.length += delta;
    }

    pub fn content<'a>(&self, input: &'a [u8]) -> &'a str {
        let slice = &input[self.start..self.start + self.length];
        unsafe { std::str::from_utf8_unchecked(slice) }
    }

    pub fn next_index_after_range(&self) -> usize {
        self.start + self.length
    }

    pub fn next_char_after_range(&self, input: &[u8]) -> u8 {
        input[self.next_index_after_range()]
    }
}
