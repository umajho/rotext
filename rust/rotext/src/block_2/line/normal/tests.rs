mod for_fn_parse {
    use crate::{
        block_2::{
            branch::surrounded::table,
            line::normal::{parse, AtxClosing, End, EndCondition, TableRelated},
            test_support::mocks::MockCursorContext,
        },
        common::m,
        events::{NewLine, VerbatimEscaping},
        types::LineNumber,
    };

    fn test(
        input: &[u8],
        end_condition: EndCondition,
        content_before: usize,
        expected_content: &[u8],
        expected_end: End,
        expected_ctx: MockCursorContext,
    ) {
        let mut ctx = MockCursorContext {
            cursor: 0,
            current_line: LineNumber::new(1),
        };
        let (actual_range, actual_end) = parse(input, &mut ctx, end_condition, content_before);
        assert_eq!(
            (expected_content, expected_end, ctx),
            (&input[actual_range], actual_end, expected_ctx)
        );
    }

    #[test]
    fn it_can_process_common_ends() {
        let end_condition = EndCondition {
            on_atx_closing: None,
            on_table_related: None,
        };

        test(
            b"",
            end_condition.clone(),
            0,
            b"",
            End::Eof,
            MockCursorContext {
                cursor: 0,
                current_line: LineNumber::new(1),
            },
        );
        test(
            b"foo",
            end_condition.clone(),
            0,
            b"foo",
            End::Eof,
            MockCursorContext {
                cursor: 3,
                current_line: LineNumber::new(1),
            },
        );
        test(
            b"foo bar",
            end_condition.clone(),
            0,
            b"foo bar",
            End::Eof,
            MockCursorContext {
                cursor: 7,
                current_line: LineNumber::new(1),
            },
        );
        for input in [&b"foo\rline 2"[..], &b"foo\nline 2"[..]] {
            test(
                input,
                end_condition.clone(),
                0,
                b"foo",
                NewLine {
                    line_after: LineNumber::new(2),
                }
                .into(),
                MockCursorContext {
                    cursor: 4,
                    current_line: LineNumber::new(2),
                },
            );
        }
        test(
            b"foo\r\nline 2",
            end_condition.clone(),
            0,
            b"foo",
            NewLine {
                line_after: LineNumber::new(2),
            }
            .into(),
            MockCursorContext {
                cursor: 5,
                current_line: LineNumber::new(2),
            },
        );
        test(
            b"==foo",
            end_condition.clone(),
            2,
            b"==foo",
            End::Eof,
            MockCursorContext {
                cursor: 5,
                current_line: LineNumber::new(1),
            },
        );
    }

    #[test]
    fn it_can_process_verbatim_escaping_ends() {
        let end_condition = EndCondition {
            on_atx_closing: None,
            on_table_related: None,
        };

        for input in [&b"<`VE`>"[..], &b"<`VE`>after"[..]] {
            test(
                input,
                end_condition.clone(),
                0,
                b"",
                VerbatimEscaping {
                    content: 2..4,
                    is_closed_forcedly: false,
                    line_after: LineNumber::new(1),
                }
                .into(),
                MockCursorContext {
                    cursor: 6,
                    current_line: LineNumber::new(1),
                },
            );
        }
        test(
            b"foo<`VE`>",
            end_condition.clone(),
            0,
            b"foo",
            VerbatimEscaping {
                content: 5..7,
                is_closed_forcedly: false,
                line_after: LineNumber::new(1),
            }
            .into(),
            MockCursorContext {
                cursor: 9,
                current_line: LineNumber::new(1),
            },
        );
        test(
            b"foo<`V\nE`>",
            end_condition.clone(),
            0,
            b"foo",
            VerbatimEscaping {
                content: 5..8,
                is_closed_forcedly: false,
                line_after: LineNumber::new(2),
            }
            .into(),
            MockCursorContext {
                cursor: 10,
                current_line: LineNumber::new(2),
            },
        );
    }

    #[test]
    fn it_cannot_process_atx_closing_ends_if_not_enabled() {
        let end_condition = EndCondition {
            on_atx_closing: None,
            on_table_related: None,
        };

        test(
            b"foo ==",
            end_condition.clone(),
            0,
            b"foo ==",
            End::Eof,
            MockCursorContext {
                cursor: 6,
                current_line: LineNumber::new(1),
            },
        );
    }

    #[test]
    fn it_can_process_atx_closing_ends_followed_by_common_end() {
        let end_condition = EndCondition {
            on_atx_closing: Some(AtxClosing {
                character: m!('='),
                count: 2,
            }),
            on_table_related: None,
        };

        test(
            b"foo ==",
            end_condition.clone(),
            0,
            b"foo",
            End::Eof,
            MockCursorContext {
                cursor: 6,
                current_line: LineNumber::new(1),
            },
        );
        test(
            b"foo ==   ",
            end_condition.clone(),
            0,
            b"foo",
            End::Eof,
            MockCursorContext {
                cursor: 9,
                current_line: LineNumber::new(1),
            },
        );
        test(
            b"foo ==\n",
            end_condition.clone(),
            0,
            b"foo",
            NewLine {
                line_after: LineNumber::new(2),
            }
            .into(),
            MockCursorContext {
                cursor: 7,
                current_line: LineNumber::new(2),
            },
        );
        test(
            b"foo ==   \n",
            end_condition.clone(),
            0,
            b"foo",
            NewLine {
                line_after: LineNumber::new(2),
            }
            .into(),
            MockCursorContext {
                cursor: 10,
                current_line: LineNumber::new(2),
            },
        );
        test(
            b"foo==",
            end_condition.clone(),
            0,
            b"foo==",
            End::Eof,
            MockCursorContext {
                cursor: 5,
                current_line: LineNumber::new(1),
            },
        );
        test(
            b"foo =",
            end_condition.clone(),
            0,
            b"foo =",
            End::Eof,
            MockCursorContext {
                cursor: 5,
                current_line: LineNumber::new(1),
            },
        );
        test(
            b"foo ===",
            end_condition.clone(),
            0,
            b"foo ===",
            End::Eof,
            MockCursorContext {
                cursor: 7,
                current_line: LineNumber::new(1),
            },
        );
        test(
            b"foo == ==",
            end_condition.clone(),
            0,
            b"foo ==",
            End::Eof,
            MockCursorContext {
                cursor: 9,
                current_line: LineNumber::new(1),
            },
        );
    }

    #[test]
    fn it_can_process_atx_closing_ends_followed_by_verbatim_escaping_end() {
        let end_condition = EndCondition {
            on_atx_closing: Some(AtxClosing {
                character: m!('='),
                count: 2,
            }),
            on_table_related: None,
        };

        test(
            b"foo ==<`VE`>",
            end_condition.clone(),
            0,
            b"foo ==",
            VerbatimEscaping {
                content: 8..10,
                is_closed_forcedly: false,
                line_after: LineNumber::new(1),
            }
            .into(),
            MockCursorContext {
                cursor: 12,
                current_line: LineNumber::new(1),
            },
        );
    }

    #[test]
    fn it_cannot_process_table_related_ends_if_not_enabled() {
        let end_condition = EndCondition {
            on_atx_closing: None,
            on_table_related: None,
        };

        test(
            b"|}",
            end_condition.clone(),
            0,
            b"|}",
            End::Eof,
            MockCursorContext {
                cursor: 2,
                current_line: LineNumber::new(1),
            },
        );
    }

    #[test]
    fn it_can_process_table_related_ends_not_concerning_captions() {
        let end_condition = EndCondition {
            on_atx_closing: None,
            on_table_related: Some(TableRelated {
                is_caption_applicable: false,
            }),
        };

        test(
            b"|}",
            end_condition.clone(),
            0,
            b"",
            table::TableRelatedEnd::TableClosing.into(),
            MockCursorContext {
                cursor: 2,
                current_line: LineNumber::new(1),
            },
        );
        test(
            b"foo|}",
            end_condition.clone(),
            0,
            b"foo",
            table::TableRelatedEnd::TableClosing.into(),
            MockCursorContext {
                cursor: 5,
                current_line: LineNumber::new(1),
            },
        );
        test(
            b"foo |}",
            end_condition.clone(),
            0,
            b"foo",
            table::TableRelatedEnd::TableClosing.into(),
            MockCursorContext {
                cursor: 6,
                current_line: LineNumber::new(1),
            },
        );
        test(
            b"|-",
            end_condition.clone(),
            0,
            b"",
            table::TableRelatedEnd::TableRowIndicator.into(),
            MockCursorContext {
                cursor: 2,
                current_line: LineNumber::new(1),
            },
        );
        test(
            b"!!",
            end_condition.clone(),
            0,
            b"",
            table::TableRelatedEnd::TableHeaderCellIndicator.into(),
            MockCursorContext {
                cursor: 2,
                current_line: LineNumber::new(1),
            },
        );
        test(
            b"||",
            end_condition.clone(),
            0,
            b"",
            table::TableRelatedEnd::DoublePipes.into(),
            MockCursorContext {
                cursor: 2,
                current_line: LineNumber::new(1),
            },
        );
        test(
            b"|+",
            end_condition.clone(),
            0,
            b"|+",
            End::Eof,
            MockCursorContext {
                cursor: 2,
                current_line: LineNumber::new(1),
            },
        );
    }

    #[test]
    fn it_can_process_table_caption_ends_if_applicable() {
        let end_condition = EndCondition {
            on_atx_closing: None,
            on_table_related: Some(TableRelated {
                is_caption_applicable: true,
            }),
        };

        test(
            b"|+",
            end_condition.clone(),
            0,
            b"",
            table::TableRelatedEnd::TableCaptionIndicator.into(),
            MockCursorContext {
                cursor: 2,
                current_line: LineNumber::new(1),
            },
        );
    }

    #[test]
    fn it_can_process_atx_closing_ends_followed_by_table_related_end() {
        let end_condition = EndCondition {
            on_atx_closing: Some(AtxClosing {
                character: m!('='),
                count: 2,
            }),
            on_table_related: Some(TableRelated {
                is_caption_applicable: true,
            }),
        };

        test(
            b"foo ==|}",
            end_condition.clone(),
            0,
            b"foo",
            table::TableRelatedEnd::TableClosing.into(),
            MockCursorContext {
                cursor: 8,
                current_line: LineNumber::new(1),
            },
        );
        test(
            b"foo == |}",
            end_condition.clone(),
            0,
            b"foo",
            table::TableRelatedEnd::TableClosing.into(),
            MockCursorContext {
                cursor: 9,
                current_line: LineNumber::new(1),
            },
        );
        test(
            b"foo ==|-",
            end_condition.clone(),
            0,
            b"foo",
            table::TableRelatedEnd::TableRowIndicator.into(),
            MockCursorContext {
                cursor: 8,
                current_line: LineNumber::new(1),
            },
        );
    }
}
