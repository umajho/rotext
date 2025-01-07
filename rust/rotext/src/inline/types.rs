use crate::{events::Event, types::Tym};

pub trait YieldContext {
    /// `ev` 是属于 `Inline` 分组的事件。
    #[must_use]
    fn r#yield(&mut self, ev: Event) -> Tym<1>;
}

#[derive(Debug, PartialEq, Eq)]
pub struct Cursor(usize);

impl Cursor {
    pub fn new(value: usize) -> Self {
        Self(value)
    }

    pub fn value(&self) -> usize {
        self.0
    }

    pub fn set_value(&mut self, value: usize) {
        self.0 = value
    }

    pub fn move_forward(&mut self, n: usize) {
        self.0 += n;
    }
}
