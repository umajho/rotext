use super::array_queue::ArrayQueue;

pub struct Peekable<const N: usize, T: Iterator> {
    iter: T,
    peeked_queue: ArrayQueue<N, Option<T::Item>>,
}

impl<const N: usize, T: Iterator> Peekable<N, T> {
    pub fn new(iter: T) -> Self {
        Self {
            iter,
            peeked_queue: ArrayQueue::new(),
        }
    }

    pub fn take_inner(self) -> T {
        self.iter
    }

    /// `index` 以 0 开始。
    pub fn peek(&mut self, index: usize) -> Option<&T::Item> {
        debug_assert!(index < N);
        while self.peeked_queue.len() < index + 1 {
            self.peeked_queue.push_back(self.iter.next());
        }
        self.peeked_queue.get(index).unwrap().as_ref()
    }
}

impl<const N: usize, T: Iterator> Iterator for Peekable<N, T> {
    type Item = T::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(item) = self.peeked_queue.pop_front() {
            return item;
        }
        self.iter.next()
    }
}
