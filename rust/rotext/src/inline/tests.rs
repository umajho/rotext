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
    use crate::inline::{
        advance_until_dicexp_will_be_ended, test_support::mocks::MockCursorContext,
    };

    fn test(input: &[u8], expected_ctx: MockCursorContext) {
        let mut ctx = MockCursorContext { cursor: 0 };
        advance_until_dicexp_will_be_ended(input, &mut ctx);
        assert_eq!(expected_ctx, ctx)
    }

    #[test]
    fn it_works() {
        for text in [&b""[..], &b"]"[..], &b"]..."[..]] {
            test(text, MockCursorContext { cursor: 0 });
        }

        for text in [&b"d100"[..], &b"d100]"[..], &b"d100]..."[..]] {
            let cursor = "d100".len();
            test(text, MockCursorContext { cursor });
        }

        for text in [&b"[]]"[..], &b"...[...]...]"[..]] {
            let cursor = text.len() - 1;
            test(text, MockCursorContext { cursor });
        }
    }
}
