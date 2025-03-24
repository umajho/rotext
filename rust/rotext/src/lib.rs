#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(proc_macro_hygiene)]
#![feature(stmt_expr_attributes)]

pub mod compiling;

pub(crate) mod utils;

pub use rotext_core::{Error as ParseError, Event, Result};

pub use compiling::{
    CompiledItem, Error as CompilationError, NewCompileOptions as CompileOption,
    Restrictions as CompileRestrictions, TagNameMap,
};

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

pub fn compile<'a>(
    input: &'a [u8],
    parsed: &[Event],
    opts: &'a CompileOption,
) -> compiling::Result<Vec<CompiledItem<'a>>> {
    let compiler = compiling::HtmlCompiler::new(opts);
    compiler.compile(input, parsed)
}

pub struct RenderOptions<'a> {
    /// XXX: 调用者需自行确保各标签的名称不会导致 XSS。
    pub tag_name_map: &'a TagNameMap<'a>,
}

pub fn render(compiled: &[CompiledItem], opts: RenderOptions) -> Vec<u8> {
    let mut buf: Vec<u8> = Vec::new();

    for item in compiled {
        match item {
            CompiledItem::Rendered(rendered) => buf.extend_from_slice(rendered),
            CompiledItem::BlockTransclusion(block_call)
            | CompiledItem::BlockExtension(block_call) => {
                let what = if matches!(item, CompiledItem::BlockTransclusion(_)) {
                    &b"transclusion"[..]
                } else {
                    &b"extension"[..]
                };

                buf.push(b'<');
                buf.extend(opts.tag_name_map.block_call_error);
                #[cfg(feature = "block-id")]
                {
                    buf.extend(b" data-block-id=\"");
                    crate::utils::write_usize(&mut buf, block_call.block_id.value());
                    buf.push(b'"');
                }
                buf.extend(b" call-type=\"");
                buf.extend(what);
                buf.extend(b"\" call-name=\"");
                utils::write_escaped_double_quoted_attribute_value(&mut buf, block_call.name);
                buf.extend(b"\" error-type=\"TODO\"></");
                buf.extend(opts.tag_name_map.block_call_error);
                buf.push(b'>');
            }
        }
    }

    buf
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
