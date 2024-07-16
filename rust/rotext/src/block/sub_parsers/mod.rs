use super::{context::Context, Event};
use crate::global;

pub mod paragraph;

mod content;
mod utils;

pub trait SubParser<'a, I: 'a + Iterator<Item = global::Event>> {
    fn next(&mut self) -> Option<Event>;

    fn take_context(&mut self) -> Box<Context<'a, I>>;
}
