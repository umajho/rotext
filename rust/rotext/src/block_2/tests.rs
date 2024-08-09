use std::marker::PhantomData;

use crate::{
    test_suites,
    utils::stack::{Stack, VecStack},
};

use super::{Parser, StackEntry};

struct Context<TStack: Stack<StackEntry>> {
    phantom_stack: PhantomData<TStack>,
}
impl<TStack: Stack<StackEntry>> Context<TStack> {
    fn new() -> Self {
        Self {
            phantom_stack: PhantomData::<TStack>,
        }
    }
}
impl<TStack: Stack<StackEntry>> test_suites::block::Context for Context<TStack> {
    fn parse(input: &str) -> impl Iterator<Item = crate::Result<crate::BlockEvent>> {
        let block_parser: Parser<TStack> = Parser::new(input.as_bytes());

        block_parser
    }
}

#[test]
fn it_works() {
    let ctx: Context<VecStack<_>> = Context::new();
    test_suites::block::run(&ctx);
}

// TODO!!: 在删掉旧版的 [crate::block] 时，不要忘了把其测试中的
// `it_works_with_array_stack` 粘贴过来。
