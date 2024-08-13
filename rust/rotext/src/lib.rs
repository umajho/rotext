#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

mod blend;
mod block_2;
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

use block_2::StackEntry;
use utils::stack::{Stack, VecStack};

pub fn parse(
    input: &[u8],
) -> blend::BlockEventStreamInlineSegmentMapper<block_2::Parser<VecStack<StackEntry>>> {
    let block_parser = block_2::Parser::new(input);

    blend::BlockEventStreamInlineSegmentMapper::new(block_parser)
}

pub fn parse_with_stack<TStackForBlockPhase: Stack<StackEntry>>(
    input: &[u8],
) -> blend::BlockEventStreamInlineSegmentMapper<block_2::Parser<TStackForBlockPhase>> {
    let block_parser = block_2::Parser::new(input);

    blend::BlockEventStreamInlineSegmentMapper::new(block_parser)
}
