use crate::{test_suites, Event};

struct Context;
impl Context {
    fn new() -> Self {
        Self
    }
}
impl test_suites::blend::Context for Context {
    /// 返回的事件都属于 `Blend` 分组。
    fn parse(input: &str) -> impl Iterator<Item = crate::Result<Event>> {
        crate::parse(input.as_bytes())
    }
}

#[test]
fn it_works() {
    let ctx = Context::new();
    test_suites::blend::run(&ctx);
}
