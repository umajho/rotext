#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(proc_macro_hygiene)]
#![feature(stmt_expr_attributes)]

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

pub use events::Event;
pub use rendering::{HtmlRenderer, NewHtmlRendererOptions, SimpleHtmlRenderer};
pub use types::{Error, Result};

use utils::stack::{Stack, VecStack};

pub fn parse(
    input: &[u8],
) -> blend::BlockEventStreamInlineSegmentMapper<
    block::Parser<VecStack<block::StackEntry>>,
    VecStack<inline::StackEntry>,
> {
    let block_parser = block::Parser::new(input);

    blend::BlockEventStreamInlineSegmentMapper::new(input, block_parser)
}

pub fn parse_with_stack<
    TBlockStack: Stack<block::StackEntry>,
    TInlineStack: Stack<inline::StackEntry>,
>(
    input: &[u8],
) -> blend::BlockEventStreamInlineSegmentMapper<block::Parser<TBlockStack>, TInlineStack> {
    let block_parser = block::Parser::new(input);

    blend::BlockEventStreamInlineSegmentMapper::new(input, block_parser)
}
