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

pub struct ArrayQueue<const N: usize, T> {
    queue: [Option<T>; N],
    end: usize,
    length: usize,
}
impl<const N: usize, T> ArrayQueue<N, T> {
    pub fn new() -> Self {
        Self {
            queue: std::array::from_fn(|_| None),
            end: 0,
            length: 0,
        }
    }

    pub fn push_back(&mut self, item: T) {
        self.length += 1;
        if self.length > N {
            unreachable!();
        }
        self.queue[self.end] = Some(item);
        self.end = (self.end + 1) % N;
    }

    pub fn pop_front(&mut self) -> Option<T> {
        if self.length == 0 {
            return None;
        }
        let item = self.queue[(self.end + N - self.length) % N].take();
        self.length -= 1;
        Some(item.unwrap())
    }
}

macro_rules! match_pop_block_id {
    ($ctx:expr, Some($id_ident:ident) => $some:block, None => $none:block,) => {
        let result;
        #[cfg(feature = "block-id")]
        {
            let $id_ident = $ctx.pop_block_id();
            result = $some;
        }
        #[cfg(not(feature = "block-id"))]
        {
            result = $none;
        }
        result
    };
}

pub(crate) use match_pop_block_id;
