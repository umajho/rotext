mod for_fn_parse {
    use crate::{
        block::{
            line::global_phase::{parse, Output},
            test_support::mocks::MockCursorContext,
        },
        events::VerbatimEscaping,
        types::LineNumber,
    };

    fn test(input: &[u8], expected: Option<Output>, expected_ctx: MockCursorContext) {
        let mut ctx = MockCursorContext {
            cursor: 0,
            current_line: LineNumber::new(1),
        };
        let actual = parse(input, &mut ctx, input[0]);
        assert_eq!((expected, expected_ctx), (actual, ctx));
    }

    #[test]
    fn it_works() {
        test(
            b"<`foo`>",
            Some(
                VerbatimEscaping {
                    content: 2..5,
                    is_closed_forcedly: false,
                    line_after: LineNumber::new(1),
                }
                .into(),
            ),
            MockCursorContext {
                cursor: 7,
                current_line: LineNumber::new(1),
            },
        );
    }

    #[test]
    fn it_works_when_there_is_no_match() {
        test(
            b"foo",
            None,
            MockCursorContext {
                cursor: 0,
                current_line: LineNumber::new(1),
            },
        );
        test(
            b"<foo",
            None,
            MockCursorContext {
                cursor: 0,
                current_line: LineNumber::new(1),
            },
        );
    }
}

mod for_fn_parse_verbatim_escaping {
    use crate::{
        block::{
            line::global_phase::parse_verbatim_escaping, test_support::mocks::MockCursorContext,
        },
        events::VerbatimEscaping,
        types::LineNumber,
    };

    fn test(input: &[u8], expected: VerbatimEscaping, expected_ctx: MockCursorContext) {
        let mut ctx = MockCursorContext {
            cursor: 0,
            current_line: LineNumber::new(1),
        };
        let actual = parse_verbatim_escaping(input, &mut ctx);
        assert_eq!((expected, expected_ctx), (actual, ctx));
    }

    #[test]
    fn it_works() {
        test(
            b"foo`>",
            VerbatimEscaping {
                content: 0..3,
                is_closed_forcedly: false,
                line_after: LineNumber::new(1),
            },
            MockCursorContext {
                cursor: 5,
                current_line: LineNumber::new(1),
            },
        );
        test(
            b"foo`>_",
            VerbatimEscaping {
                content: 0..3,
                is_closed_forcedly: false,
                line_after: LineNumber::new(1),
            },
            MockCursorContext {
                cursor: 5,
                current_line: LineNumber::new(1),
            },
        );
    }

    #[test]
    fn it_works_with_spaces_at_edge() {
        test(
            b" foo `>",
            VerbatimEscaping {
                content: 1..4,
                is_closed_forcedly: false,
                line_after: LineNumber::new(1),
            },
            MockCursorContext {
                cursor: 7,
                current_line: LineNumber::new(1),
            },
        );
        test(
            b"foo `>",
            VerbatimEscaping {
                content: 0..3,
                is_closed_forcedly: false,
                line_after: LineNumber::new(1),
            },
            MockCursorContext {
                cursor: 6,
                current_line: LineNumber::new(1),
            },
        );
        test(
            b" foo`>",
            VerbatimEscaping {
                content: 1..4,
                is_closed_forcedly: false,
                line_after: LineNumber::new(1),
            },
            MockCursorContext {
                cursor: 6,
                current_line: LineNumber::new(1),
            },
        );
        test(
            b"  foo  `>",
            VerbatimEscaping {
                content: 1..6,
                is_closed_forcedly: false,
                line_after: LineNumber::new(1),
            },
            MockCursorContext {
                cursor: 9,
                current_line: LineNumber::new(1),
            },
        );
    }

    #[test]
    fn it_works_with_multiple_backticks() {
        test(
            b"`foo``>",
            VerbatimEscaping {
                content: 1..4,
                is_closed_forcedly: false,
                line_after: LineNumber::new(1),
            },
            MockCursorContext {
                cursor: 7,
                current_line: LineNumber::new(1),
            },
        );
        test(
            b"` ` ``>",
            VerbatimEscaping {
                content: 2..3,
                is_closed_forcedly: false,
                line_after: LineNumber::new(1),
            },
            MockCursorContext {
                cursor: 7,
                current_line: LineNumber::new(1),
            },
        );
    }

    #[test]
    fn it_works_when_closing_part_is_missed() {
        test(
            b"",
            VerbatimEscaping {
                content: 0..0,
                is_closed_forcedly: true,
                line_after: LineNumber::new(1),
            },
            MockCursorContext {
                cursor: 0,
                current_line: LineNumber::new(1),
            },
        );
        test(
            b"foo",
            VerbatimEscaping {
                content: 0..3,
                is_closed_forcedly: true,
                line_after: LineNumber::new(1),
            },
            MockCursorContext {
                cursor: 3,
                current_line: LineNumber::new(1),
            },
        );
        test(
            b"foo ",
            VerbatimEscaping {
                content: 0..4,
                is_closed_forcedly: true,
                line_after: LineNumber::new(1),
            },
            MockCursorContext {
                cursor: 4,
                current_line: LineNumber::new(1),
            },
        );
        test(
            b"foo`",
            VerbatimEscaping {
                content: 0..4,
                is_closed_forcedly: true,
                line_after: LineNumber::new(1),
            },
            MockCursorContext {
                cursor: 4,
                current_line: LineNumber::new(1),
            },
        );
        test(
            b"`foo`>",
            VerbatimEscaping {
                content: 1..6,
                is_closed_forcedly: true,
                line_after: LineNumber::new(1),
            },
            MockCursorContext {
                cursor: 6,
                current_line: LineNumber::new(1),
            },
        );
    }

    #[test]
    fn it_works_with_multilines() {
        test(
            b"foo\r`>",
            VerbatimEscaping {
                content: 0..4,
                is_closed_forcedly: false,
                line_after: LineNumber::new(2),
            },
            MockCursorContext {
                cursor: 6,
                current_line: LineNumber::new(2),
            },
        );
        test(
            b"foo\n`>",
            VerbatimEscaping {
                content: 0..4,
                is_closed_forcedly: false,
                line_after: LineNumber::new(2),
            },
            MockCursorContext {
                cursor: 6,
                current_line: LineNumber::new(2),
            },
        );
        test(
            b"foo\r\n`>",
            VerbatimEscaping {
                content: 0..5,
                is_closed_forcedly: false,
                line_after: LineNumber::new(2),
            },
            MockCursorContext {
                cursor: 7,
                current_line: LineNumber::new(2),
            },
        );
        test(
            b"``foo`\n```>",
            VerbatimEscaping {
                content: 2..7,
                is_closed_forcedly: false,
                line_after: LineNumber::new(2),
            },
            MockCursorContext {
                cursor: 11,
                current_line: LineNumber::new(2),
            },
        );
    }
}
