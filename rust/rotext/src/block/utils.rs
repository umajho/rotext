use super::global_mapper;

#[derive(Clone, Copy)]
pub struct InputCursor(Option<usize>);

impl InputCursor {
    pub fn new() -> InputCursor {
        InputCursor(None)
    }

    pub fn value(&self) -> Option<usize> {
        self.0
    }

    pub fn apply(&mut self, mapped: &global_mapper::Mapped) {
        match mapped {
            &global_mapper::Mapped::CharAt(new_cursor) => self.0 = Some(new_cursor),
            global_mapper::Mapped::NextChar => match self.0 {
                Some(ref mut cursor) => *cursor += 1,
                None => unreachable!(),
            },
            _ => self.0 = None,
        }
    }

    pub fn applying(&self, mapped: &global_mapper::Mapped) -> Self {
        let mut copied = *self;
        copied.apply(mapped);
        copied
    }

    pub fn at(&self, input: &[u8]) -> Option<u8> {
        self.0.map(|cursor| input[cursor])
    }
}

pub struct Peekable<const N: usize, I: Iterator> {
    inner: I,

    buffer: [Option<I::Item>; N],
    start: usize,
}

impl<const N: usize, I: Iterator> Peekable<N, I> {
    pub fn new(inner: I) -> Peekable<N, I> {
        Peekable {
            inner,
            buffer: std::array::from_fn(|_| None),
            start: 0,
        }
    }

    /// `i` 以 0 为起始。
    pub fn peek(&mut self, i: usize) -> Option<&I::Item> {
        debug_assert!(i < N);
        if self.buffer[(self.start + i) % N].is_none() {
            if i > 0 {
                self.peek(i - 1);
            }
            self.buffer[(self.start + i) % N] = self.inner.next();
        }
        self.buffer[(self.start + i) % N].as_ref()
    }
}

impl<const N: usize, I: Iterator> Iterator for Peekable<N, I> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let taken = self.buffer[self.start].take();
        if taken.is_some() {
            self.start = (self.start + 1) % 3;
            taken
        } else {
            self.inner.next()
        }
    }
}
