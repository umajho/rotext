mod blend;
mod block;
mod common;
mod errors;
mod events;
mod global;
mod inline;
mod types;

pub mod rendering;
pub mod utils;

#[cfg(test)]
pub(crate) mod test_utils;

use block::StackEntry;
pub use errors::{Error, Result};
pub use events::{BlendEvent, BlockEvent, Event};
pub use rendering::{HtmlRenderer, NewHtmlRendererOptoins};
use utils::stack::{Stack, VecStack};

pub fn parse(
    input: &[u8],
) -> blend::BlockEventStreamInlineSegmentMapper<block::Parser<VecStack<StackEntry>>> {
    let global_parser = global::Parser::new(input, global::NewParserOptions::default());
    let block_parser = block::Parser::new(input, global_parser);

    blend::BlockEventStreamInlineSegmentMapper::new(block_parser)
}

pub fn parse_with_stack<TStackForBlockPhase: Stack<StackEntry>>(
    input: &[u8],
) -> blend::BlockEventStreamInlineSegmentMapper<block::Parser<TStackForBlockPhase>> {
    let global_parser = global::Parser::new(input, global::NewParserOptions::default());
    let block_parser = block::Parser::new(input, global_parser);

    blend::BlockEventStreamInlineSegmentMapper::new(block_parser)
}
