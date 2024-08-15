mod for_fn_parse {
    use crate::{
        block::{
            line::verbatim::{parse, AtLineBeginning, End, EndCondition, Fence},
            test_support::mocks::MockCursorContext,
        },
        types::LineNumber,
    };

    struct TestOptions<'a> {
        input: &'a [u8],
        cursor: usize,
        end_condition: EndCondition,
        spaces_before: usize,
        at_line_beginning: Option<AtLineBeginning>,
        expected_content: &'a [u8],
        expected_end: End,
        expected_ctx: MockCursorContext,
    }

    fn test(opts: TestOptions) {
        let mut ctx = MockCursorContext {
            cursor: opts.cursor,
            current_line: LineNumber::new_universal(1),
        };
        let (actual_range, actual_end) = parse(
            opts.input,
            &mut ctx,
            opts.end_condition,
            opts.spaces_before,
            opts.at_line_beginning,
        );
        assert_eq!(
            (opts.expected_content, opts.expected_end, ctx),
            (&opts.input[actual_range], actual_end, opts.expected_ctx)
        );
    }

    #[test]
    fn it_can_process_common_ends() {
        let end_condition = EndCondition { on_fence: None };

        test(TestOptions {
            input: b"",
            cursor: 0,
            end_condition: end_condition.clone(),
            spaces_before: 0,
            at_line_beginning: None,
            expected_content: b"",
            expected_end: End::Eof,
            expected_ctx: MockCursorContext {
                cursor: 0,
                current_line: LineNumber::new_universal(1),
            },
        });

        test(TestOptions {
            input: b"foo",
            cursor: 0,
            end_condition: end_condition.clone(),
            spaces_before: 0,
            at_line_beginning: None,
            expected_content: b"foo",
            expected_end: End::Eof,
            expected_ctx: MockCursorContext {
                cursor: 3,
                current_line: LineNumber::new_universal(1),
            },
        });

        test(TestOptions {
            input: b"  foo",
            cursor: 2,
            end_condition: end_condition.clone(),
            spaces_before: 2,
            at_line_beginning: None,
            expected_content: b"  foo",
            expected_end: End::Eof,
            expected_ctx: MockCursorContext {
                cursor: 5,
                current_line: LineNumber::new_universal(1),
            },
        });

        test(TestOptions {
            input: b"  foo",
            cursor: 2,
            end_condition: end_condition.clone(),
            spaces_before: 2,
            at_line_beginning: Some(AtLineBeginning { indent: 0 }),
            expected_content: b"  foo",
            expected_end: End::Eof,
            expected_ctx: MockCursorContext {
                cursor: 5,
                current_line: LineNumber::new_universal(1),
            },
        });

        test(TestOptions {
            input: b"  foo",
            cursor: 2,
            end_condition: end_condition.clone(),
            spaces_before: 2,
            at_line_beginning: Some(AtLineBeginning { indent: 1 }),
            expected_content: b" foo",
            expected_end: End::Eof,
            expected_ctx: MockCursorContext {
                cursor: 5,
                current_line: LineNumber::new_universal(1),
            },
        });

        for indent in [2, 3] {
            test(TestOptions {
                input: b"  foo",
                cursor: 2,
                end_condition: end_condition.clone(),
                spaces_before: 2,
                at_line_beginning: Some(AtLineBeginning { indent }),
                expected_content: b"foo",
                expected_end: End::Eof,
                expected_ctx: MockCursorContext {
                    cursor: 5,
                    current_line: LineNumber::new_universal(1),
                },
            });
        }
    }

    #[test]
    fn it_cannot_process_fence_ends_if_not_enabled() {
        let end_condition = EndCondition { on_fence: None };

        test(TestOptions {
            input: b"```",
            cursor: 0,
            end_condition: end_condition.clone(),
            spaces_before: 0,
            at_line_beginning: Some(AtLineBeginning { indent: 0 }),
            expected_content: b"```",
            expected_end: End::Eof,
            expected_ctx: MockCursorContext {
                cursor: 3,
                current_line: LineNumber::new_universal(1),
            },
        });
    }

    #[test]
    fn it_can_process_fence_ends() {
        let end_condition = EndCondition {
            on_fence: Some(Fence {
                character: b'`',
                minimum_count: 3,
            }),
        };

        test(TestOptions {
            input: b"```",
            cursor: 0,
            end_condition: end_condition.clone(),
            spaces_before: 0,
            at_line_beginning: Some(AtLineBeginning { indent: 0 }),
            expected_content: b"",
            expected_end: End::Fence,
            expected_ctx: MockCursorContext {
                cursor: 3,
                current_line: LineNumber::new_universal(1),
            },
        });

        test(TestOptions {
            input: b"```",
            cursor: 0,
            end_condition: end_condition.clone(),
            spaces_before: 0,
            at_line_beginning: None,
            expected_content: b"```",
            expected_end: End::Eof,
            expected_ctx: MockCursorContext {
                cursor: 3,
                current_line: LineNumber::new_universal(1),
            },
        });

        for indent in 0..=3 {
            test(TestOptions {
                input: b"  ```",
                cursor: 2,
                end_condition: end_condition.clone(),
                spaces_before: 0,
                at_line_beginning: Some(AtLineBeginning { indent }),
                expected_content: b"",
                expected_end: End::Fence,
                expected_ctx: MockCursorContext {
                    cursor: 5,
                    current_line: LineNumber::new_universal(1),
                },
            });
        }
    }
}