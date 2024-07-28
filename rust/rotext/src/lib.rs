mod blend;
mod block;
mod common;
mod events;
mod global;
mod inline;
pub mod rendering;

#[cfg(test)]
pub(crate) mod test_utils;

pub use events::{BlendEvent, Event};
pub use rendering::{HtmlRenderer, NewHtmlRendererOptoins};

pub fn parse(input: &[u8]) -> blend::BlockEventStreamInlineSegmentMapper {
    let global_parser = global::Parser::new(input, global::NewParserOptions::default());
    let block_parser = block::Parser::new(input, global_parser);

    blend::BlockEventStreamInlineSegmentMapper::new(block_parser, Box::new(inline::Parser::new))
}
