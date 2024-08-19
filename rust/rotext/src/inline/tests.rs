use crate::{events::InlineEvent, test_suites, BlendEvent, Event};

mod for_fn_advance_until_potential_ref_link_content_ends {
    use crate::inline::{
        advance_until_potential_ref_link_content_ends, test_support::mocks::MockCursorContext,
    };

    fn test(input: &[u8], expected: Option<()>, expected_ctx: MockCursorContext) {
        let mut ctx = MockCursorContext { cursor: 0 };
        let actual = advance_until_potential_ref_link_content_ends(input, &mut ctx);
        assert_eq!((expected, expected_ctx), (actual, ctx))
    }

    #[test]
    fn it_works() {
        for text in [&b""[..], &b" "[..], &b"?"[..]] {
            test(text, None, MockCursorContext { cursor: 0 });
        }
        for text in [&b"TP"[..], &b"TP."[..]] {
            test(text, None, MockCursorContext { cursor: text.len() });
        }
        for text in [&b"TP. "[..], &b"TP.?"[..]] {
            let cursor = text.len() - 1;
            test(text, None, MockCursorContext { cursor });
        }

        for text in [
            &b"TP.abc"[..],
            &b"TP.abc "[..],
            &b"TP.abc1"[..],
            &b"TP.abc?"[..],
            &b"TP.abc#"[..],
            &b"TP.abc# "[..],
            &b"TP.abc#a"[..],
            &b"TP.abc#?"[..],
        ] {
            test(text, Some(()), MockCursorContext { cursor: 6 });
        }

        for text in [
            &b"TP.abc#123"[..],
            &b"TP.abc#123 "[..],
            &b"TP.abc#123a"[..],
            &b"TP.abc#123?"[..],
        ] {
            test(text, Some(()), MockCursorContext { cursor: 10 });
        }

        for text in [
            &b"TP.456"[..],
            &b"TP.456 "[..],
            &b"TP.456a"[..],
            &b"TP.456?"[..],
            &b"TP.456#123"[..],
        ] {
            test(text, Some(()), MockCursorContext { cursor: 6 });
        }
    }
}

mod for_fn_advance_until_dicexp_will_be_ended {
    use std::ops::Range;

    use crate::inline::{advance_until_dicexp_ends, test_support::mocks::MockCursorContext};

    fn test(input: &[u8], expected: Range<usize>, expected_ctx: MockCursorContext) {
        let mut ctx = MockCursorContext { cursor: 0 };
        let actual = advance_until_dicexp_ends(input, &mut ctx);
        assert_eq!((expected, expected_ctx), (actual, ctx))
    }

    #[test]
    fn it_works() {
        test(b"", 0..0, MockCursorContext { cursor: 0 });
        test(b"]", 0..0, MockCursorContext { cursor: 1 });
        test(b"]...", 0..0, MockCursorContext { cursor: 1 });

        test(b"d100", 0..4, MockCursorContext { cursor: 4 });
        test(b"d100]", 0..4, MockCursorContext { cursor: 5 });
        test(b"d100]...", 0..4, MockCursorContext { cursor: 5 });

        test(b"[]]", 0..2, MockCursorContext { cursor: 3 });
        test(b"[]", 0..2, MockCursorContext { cursor: 2 });
        test(b"...[...]...]", 0..11, MockCursorContext { cursor: 12 });
    }
}

struct Context;
impl Context {
    fn new() -> Self {
        Self
    }
}
impl test_suites::inline::Context for Context {
    fn parse(input: &str) -> impl Iterator<Item = crate::Result<InlineEvent>> {
        let evs: crate::Result<Vec<_>> = crate::parse(input.as_bytes()).collect();
        let evs = match evs {
            Ok(evs) => evs,
            Err(_) => todo!("should yield err!"),
        };

        let evs = if !evs.is_empty() {
            if !matches!(evs.first(), Some(BlendEvent::EnterParagraph(_))) {
                panic!("the input should be a paragraph!")
            }
            if !matches!(evs.last(), Some(BlendEvent::ExitBlock(_))) {
                unreachable!()
            }
            evs[1..evs.len() - 1].to_vec()
        } else {
            evs
        };

        if evs.iter().any(|ev| matches!(ev, BlendEvent::ExitBlock(_))) {
            panic!("the input should be ONE paragraph!")
        }

        evs.into_iter()
            .map(|ev| -> crate::Result<InlineEvent> { Ok(Event::from(ev).try_into().unwrap()) })
    }
}

#[test]
fn it_works() {
    let ctx = Context::new();
    test_suites::inline::run(&ctx);
}
