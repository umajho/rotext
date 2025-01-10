use crate::{events::is_event_of, test_suites, Event};

mod for_mod_leaf {
    mod for_mod_ref_link {
        mod for_fn_advance_until_potential_content_ends {
            use crate::inline::{
                leaf::ref_link::advance_until_potential_content_ends, types::Cursor,
            };

            fn test(input: &[u8], expected: Option<()>, expected_cursor: Cursor) {
                let mut cursor = Cursor::new(0);
                let actual = advance_until_potential_content_ends(input, &mut cursor);
                assert_eq!((expected, expected_cursor), (actual, cursor))
            }

            #[test]
            fn it_works() {
                for text in [&b""[..], &b" "[..], &b"?"[..]] {
                    test(text, None, Cursor::new(0));
                }
                for text in [&b"TP"[..], &b"TP."[..]] {
                    test(text, None, Cursor::new(text.len()));
                }
                for text in [&b"TP. "[..], &b"TP.?"[..]] {
                    test(text, None, Cursor::new(text.len() - 1));
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
                    test(text, Some(()), Cursor::new(6));
                }

                for text in [
                    &b"TP.abc#123"[..],
                    &b"TP.abc#123 "[..],
                    &b"TP.abc#123a"[..],
                    &b"TP.abc#123?"[..],
                ] {
                    test(text, Some(()), Cursor::new(10));
                }

                for text in [
                    &b"TP.456"[..],
                    &b"TP.456 "[..],
                    &b"TP.456a"[..],
                    &b"TP.456?"[..],
                    &b"TP.456#123"[..],
                ] {
                    test(text, Some(()), Cursor::new(6));
                }

                for text in [&b"#123"[..], &b"#123 "[..], &b"#123"[..], &b"#123?"[..]] {
                    test(text, Some(()), Cursor::new(4));
                }
            }
        }
    }

    mod for_mod_dicexp {
        mod for_fn_advance_until_ends {
            use std::ops::Range;

            use crate::inline::{leaf::dicexp::advance_until_ends, types::Cursor};

            fn test(input: &[u8], expected: Range<usize>, expected_cursor: Cursor) {
                let mut cursor = Cursor::new(0);
                let actual = advance_until_ends(input, &mut cursor);
                assert_eq!((expected, expected_cursor), (actual, cursor))
            }

            #[test]
            fn it_works() {
                test(b"", 0..0, Cursor::new(0));
                test(b"]", 0..0, Cursor::new(1));
                test(b"]...", 0..0, Cursor::new(1));

                test(b"d100", 0..4, Cursor::new(4));
                test(b"d100]", 0..4, Cursor::new(5));
                test(b"d100]...", 0..4, Cursor::new(5));

                test(b"[]]", 0..2, Cursor::new(3));
                test(b"[]", 0..2, Cursor::new(2));
                test(b"...[...]...]", 0..11, Cursor::new(12));
            }
        }
    }
}

struct Context;
impl Context {
    fn new() -> Self {
        Self
    }
}
impl test_suites::inline::Context for Context {
    /// 返回的事件都属于 `Inline` 分组。
    fn parse(input: &str) -> Vec<Event> {
        // [parse] 返回的结果是一系列 `Blend` 分组的事件。
        let evs: crate::Result<Vec<_>> = crate::parse(input.as_bytes()).collect();
        let evs = match evs {
            Ok(evs) => evs,
            Err(_) => todo!("should yield err!"),
        };

        let evs = if !evs.is_empty() {
            if !matches!(evs.first(), Some(Event::EnterParagraph(_))) {
                panic!("the input should be a paragraph!")
            }
            if !matches!(evs.last(), Some(Event::ExitBlock(_))) {
                unreachable!()
            }
            evs[1..evs.len() - 1].to_vec()
        } else {
            evs
        };

        if evs.iter().any(|ev| matches!(ev, Event::ExitBlock(_))) {
            panic!("the input should be ONE paragraph! input: {:?}", evs)
        }

        evs.iter()
            .for_each(|item| debug_assert!(is_event_of!(Inline, item)));
        evs
    }
}

#[test]
fn it_works() {
    let ctx = Context::new();
    test_suites::inline::run(&ctx);
}
