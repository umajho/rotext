#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(proc_macro_hygiene)]
#![feature(stmt_expr_attributes)]

pub mod compiling;
pub mod executing;

pub(crate) mod utils;

pub use rotext_core::{Error as ParseError, Event, Result};

pub use compiling::{
    CompiledItem, Error as CompilationError, NewCompileOptions as CompileOption,
    Restrictions as CompileRestrictions,
};
pub use executing::{NewExecutorOptions as ExecuteOptions, TagNameMap};

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
    let compiler = compiling::Compiler::new(opts);
    compiler.compile(input, parsed)
}

pub fn execute(
    input: &[u8],
    parsed: &[Event],
    compiled: &[CompiledItem],
    opts: &ExecuteOptions,
) -> Vec<u8> {
    let mut buf: Vec<u8> = Vec::new();
    let executor = executing::Executor::new(opts);
    executor.execute(&mut buf, input, parsed, compiled);
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
