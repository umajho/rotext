use super::{context::Context, Event};
use crate::global;

pub mod paragraph;

mod content;
mod utils;

pub enum Result {
    ToYield(Event),
    ToPauseForNewLine,
    Done,
}

pub trait SubParser<'a, I: 'a + Iterator<Item = global::Event>> {
    fn next(&mut self, ctx: &mut Context<'a, I>) -> Result;

    fn resume_from_pause_for_new_line_and_continue(&mut self);
    fn resume_from_pause_for_new_line_and_exit(&mut self);
}
