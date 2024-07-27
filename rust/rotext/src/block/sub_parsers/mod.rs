use super::context::Context;
use crate::events::{BlockEvent, NewLine};

pub mod code_block;
pub mod heading;
pub mod paragraph;

mod content;
mod utils;

#[derive(Debug)]
pub enum Result {
    ToYield(BlockEvent),
    ToPauseForNewLine,
    Done,
}

pub trait SubParser<'a> {
    fn next(&mut self, ctx: &mut Context<'a>) -> Result;

    fn resume_from_pause_for_new_line_and_continue(&mut self, new_line: NewLine);
    fn resume_from_pause_for_new_line_and_exit(&mut self);
}
