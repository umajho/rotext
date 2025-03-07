pub struct ArrayQueue<const N: usize, T> {
    queue: [Option<T>; N],
    end: usize,
    length: usize,
}
impl<const N: usize, T> ArrayQueue<N, T> {
    pub fn new() -> Self {
        Self {
            queue: core::array::from_fn(|_| None),
            end: 0,
            length: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.length
    }

    fn index(&self, index: usize) -> usize {
        (self.end + N - self.length + index) % N
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        debug_assert!(index < self.length);
        self.queue[self.index(index)].as_ref()
    }

    pub fn push_back(&mut self, item: T) {
        self.length += 1;
        debug_assert!(self.length <= N);
        self.queue[self.end] = Some(item);
        self.end = (self.end + 1) % N;
    }

    pub fn pop_front(&mut self) -> Option<T> {
        if self.length == 0 {
            return None;
        }
        let item = self.queue[self.index(0)].take();
        self.length -= 1;
        Some(item.unwrap())
    }
}
