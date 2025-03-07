#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(proc_macro_hygiene)]
#![feature(stmt_expr_attributes)]

pub mod rendering;

pub use rotext_core::{Error, Event, Result};

pub use rendering::{HtmlRenderer, NewHtmlRendererOptions};

use rotext_core::{
    BlockEventStreamInlineSegmentMapper, BlockParser, BlockStackEntry, InlineStackEntry,
};

use rotext_utils::stack::VecStack;

pub fn parse(
    input: &[u8],
) -> BlockEventStreamInlineSegmentMapper<
    BlockParser<VecStack<BlockStackEntry>>,
    VecStack<InlineStackEntry>,
> {
    let block_parser = BlockParser::new(input);

    BlockEventStreamInlineSegmentMapper::new(input, block_parser)
}

#[cfg(test)]
mod tests {
    use rotext_internal_test::{BlendContext, BlockContext, InlineContext};

    use rotext_utils::stack::VecStack;

    #[test]
    fn inline_test_suite_passes() {
        let ctx: InlineContext<VecStack<_>, VecStack<_>> = InlineContext::new();
        rotext_internal_test::suites::inline::run(&ctx);
    }

    #[test]
    fn block_test_suite_passes() {
        let ctx: BlockContext<VecStack<_>> = BlockContext::new();
        rotext_internal_test::suites::block::run(&ctx);
    }

    #[test]
    fn blend_test_suite_passes() {
        let ctx: BlendContext<VecStack<_>, VecStack<_>> = BlendContext::new();
        rotext_internal_test::suites::blend::run(&ctx);
    }
}
