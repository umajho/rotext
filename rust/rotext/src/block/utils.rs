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

pub struct Peekable3<I: Iterator> {
    inner: I,

    buffer: [Option<I::Item>; 3],
    start: usize,
}

impl<I: Iterator> Peekable3<I> {
    pub fn new(inner: I) -> Peekable3<I> {
        Peekable3 {
            inner,
            buffer: [None, None, None],
            start: 0,
        }
    }

    pub fn peek_1(&mut self) -> Option<&I::Item> {
        if self.buffer[self.start].is_none() {
            self.buffer[self.start] = self.inner.next();
        }
        self.buffer[self.start].as_ref()
    }
    pub fn peek_2(&mut self) -> Option<&I::Item> {
        if self.buffer[(self.start + 1) % 3].is_none() {
            self.peek_1();
            self.buffer[(self.start + 1) % 3] = self.inner.next();
        }
        self.buffer[(self.start + 1) % 3].as_ref()
    }
    pub fn peek_3(&mut self) -> Option<&I::Item> {
        if self.buffer[(self.start + 2) % 3].is_none() {
            self.peek_2();
            self.buffer[(self.start + 2) % 3] = self.inner.next();
        }
        self.buffer[(self.start + 2) % 3].as_ref()
    }
}

impl<I: Iterator> Iterator for Peekable3<I> {
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
