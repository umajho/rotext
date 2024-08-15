#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

mod blend;
mod block;
mod common;
mod events;
mod inline;
mod types;

pub mod rendering;
pub mod utils;

#[cfg(test)]
pub(crate) mod test_suites;
#[cfg(test)]
pub(crate) mod test_support;

pub use events::{BlendEvent, BlockEvent, Event};
pub use rendering::{HtmlRenderer, NewHtmlRendererOptoins};
pub use types::{Error, Result};

use block::StackEntry;
use utils::stack::{Stack, VecStack};

pub fn parse(
    input: &[u8],
) -> blend::BlockEventStreamInlineSegmentMapper<block::Parser<VecStack<StackEntry>>> {
    let block_parser = block::Parser::new(input);

    blend::BlockEventStreamInlineSegmentMapper::new(input, block_parser)
}

pub fn parse_with_stack<TStackForBlockPhase: Stack<StackEntry>>(
    input: &[u8],
) -> blend::BlockEventStreamInlineSegmentMapper<block::Parser<TStackForBlockPhase>> {
    let block_parser = block::Parser::new(input);

    blend::BlockEventStreamInlineSegmentMapper::new(input, block_parser)
}
