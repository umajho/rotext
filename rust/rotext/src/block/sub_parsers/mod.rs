use super::context::Context;
use crate::events::{BlockEvent, NewLine};

pub mod code_block;
pub mod heading;
pub mod paragraph;

mod content;
mod utils;

#[derive(Debug)]
pub enum Output {
    ToYield(BlockEvent),
    ToPauseForNewLine,
    Done(HaveMet),
}

#[derive(Debug, Clone, Copy)]
pub enum HaveMet {
    None,

    TableClosing,
    TableCaptionIndicator,
    TableRowIndicator,
    TableHeaderCellIndicator,
    DoublePipes,
}

pub trait SubParser<'a> {
    fn next(&mut self, ctx: &mut Context<'a>) -> Output;

    fn resume_from_pause_for_new_line_and_continue(&mut self, new_line: NewLine);
    fn resume_from_pause_for_new_line_and_exit(&mut self);
}

pub struct InTable {
    pub has_yielded_since_entered: bool,
}
