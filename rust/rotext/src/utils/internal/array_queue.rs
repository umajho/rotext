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
