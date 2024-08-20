use crate::{events::InlineEvent, types::Tym};

pub trait YieldContext {
    #[must_use]
    fn r#yield(&mut self, ev: InlineEvent) -> Tym<1>;
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

#[derive(Default)]
pub struct EndCondition {
    pub on_strong_closing: bool,
    pub on_strikethrough_closing: bool,
}
