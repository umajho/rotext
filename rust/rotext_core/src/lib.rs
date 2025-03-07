#![cfg_attr(not(test), no_std)]
#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(proc_macro_hygiene)]
#![feature(stmt_expr_attributes)]

mod blend;
mod block;
mod common;
pub mod events;
mod inline;
mod types;

mod internal_utils;

pub use events::{Event, EventType};
pub use types::{BlockId, Error, LineNumber, Result, Stack};

pub use block::StackEntry as BlockStackEntry;
pub use inline::StackEntry as InlineStackEntry;

pub use block::Parser as BlockParser;

pub use blend::BlockEventStreamInlineSegmentMapper;

pub fn parse<TBlockStack: Stack<BlockStackEntry>, TInlineStack: Stack<InlineStackEntry>>(
    input: &[u8],
) -> blend::BlockEventStreamInlineSegmentMapper<block::Parser<TBlockStack>, TInlineStack> {
    let block_parser = block::Parser::new(input);

    blend::BlockEventStreamInlineSegmentMapper::new(input, block_parser)
}
