use super::{context::Context, Event};

pub mod code_block;
pub mod heading;
pub mod paragraph;

mod content;
mod utils;

#[derive(Debug)]
pub enum Result {
    ToYield(Event),
    ToPauseForNewLine,
    Done,
}

pub trait SubParser<'a> {
    fn next(&mut self, ctx: &mut Context<'a>) -> Result;

    fn resume_from_pause_for_new_line_and_continue(&mut self);
    fn resume_from_pause_for_new_line_and_exit(&mut self);
}
