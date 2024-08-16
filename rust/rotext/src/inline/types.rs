use crate::{events::InlineEvent, types::Tym};

pub trait CursorContext {
    fn cursor(&self) -> usize;
    fn set_cursor(&mut self, value: usize);
    fn move_cursor_forward(&mut self, n: usize);
}
pub trait YieldContext {
    #[must_use]
    fn r#yield(&mut self, ev: InlineEvent) -> Tym<1>;
}
