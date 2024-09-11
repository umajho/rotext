use crate::{test_suites, BlendEvent};

struct Context;
impl Context {
    fn new() -> Self {
        Self
    }
}
impl test_suites::blend::Context for Context {
    fn parse(input: &str) -> impl Iterator<Item = crate::Result<BlendEvent>> {
        crate::parse(input.as_bytes())
    }
}

#[test]
fn it_works() {
    let ctx = Context::new();
    test_suites::blend::run(&ctx);
}
