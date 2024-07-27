mod blend;
mod block;
mod common;
mod events;
mod global;
mod inline;
pub mod rendering;

pub use events::{BlendEvent, Event};
pub use rendering::{render_to_html, RenderToHTMLOptions};

pub fn parse(input: &[u8]) -> blend::BlockEventStreamInlineSegmentMapper {
    let global_parser = global::Parser::new(input, 0);
    let block_parser = block::Parser::new(input, global_parser);

    blend::BlockEventStreamInlineSegmentMapper::new(block_parser, Box::new(inline::Parser::new))
}
