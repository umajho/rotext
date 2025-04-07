mod for_fn_parse {
    use crate::{
        block::{
            braced::table,
            line::normal::{
                AtxClosing, ContentBefore, End, EndCondition, MatchedCallNameExtraMatched,
                Matching, TableRelated, parse,
            },
            test_support::mocks::MockCursorContext,
        },
        common::m,
        events::{NewLine, VerbatimEscaping},
        internal_utils::string::is_whitespace,
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
            current_line: LineNumber::new_universal(1),
        };
        let (actual_range, actual_end) = parse(
            input,
            &mut ctx,
            end_condition,
            // TODO: [ContentBefore::Space] 的情况也应该允许测试。
            ContentBefore::NotSpace(content_before),
        );
        assert_eq!(
            (expected_content, expected_end, expected_ctx),
            (&input[actual_range], actual_end, ctx)
        );
    }

    #[test]
    fn it_can_process_common_ends() {
        let end_condition = EndCondition::default();

        test(
            b"",
            end_condition.clone(),
            0,
            b"",
            End::Eof,
            MockCursorContext {
                cursor: 0,
                current_line: LineNumber::new_universal(1),
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
                current_line: LineNumber::new_universal(1),
            },
        );
        test(
            b"foo   ",
            end_condition.clone(),
            0,
            b"foo",
            End::Eof,
            MockCursorContext {
                cursor: 6,
                current_line: LineNumber::new_universal(1),
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
                current_line: LineNumber::new_universal(1),
            },
        );
        for input in [&b"foo\rline 2"[..], &b"foo\nline 2"[..]] {
            test(
                input,
                end_condition.clone(),
                0,
                b"foo",
                NewLine {
                    line_after: LineNumber::new_universal(2),
                }
                .into(),
                MockCursorContext {
                    cursor: 4,
                    current_line: LineNumber::new_universal(2),
                },
            );
        }
        test(
            b"foo\r\nline 2",
            end_condition.clone(),
            0,
            b"foo",
            NewLine {
                line_after: LineNumber::new_universal(2),
            }
            .into(),
            MockCursorContext {
                cursor: 5,
                current_line: LineNumber::new_universal(2),
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
                current_line: LineNumber::new_universal(1),
            },
        );
    }

    #[test]
    fn it_can_process_verbatim_escaping_ends() {
        let end_condition = EndCondition::default();

        for input in [&b"<`VE`>"[..], &b"<`VE`>after"[..]] {
            test(
                input,
                end_condition.clone(),
                0,
                b"",
                VerbatimEscaping {
                    content: 2..4,
                    is_closed_forcedly: false,
                    line_after: LineNumber::new_universal(1),
                }
                .into(),
                MockCursorContext {
                    cursor: 6,
                    current_line: LineNumber::new_universal(1),
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
                line_after: LineNumber::new_universal(1),
            }
            .into(),
            MockCursorContext {
                cursor: 9,
                current_line: LineNumber::new_universal(1),
            },
        );
        test(
            b"foo   <`VE`>",
            end_condition.clone(),
            0,
            b"foo   ",
            VerbatimEscaping {
                content: 8..10,
                is_closed_forcedly: false,
                line_after: LineNumber::new_universal(1),
            }
            .into(),
            MockCursorContext {
                cursor: 12,
                current_line: LineNumber::new_universal(1),
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
                line_after: LineNumber::new_universal(2),
            }
            .into(),
            MockCursorContext {
                cursor: 10,
                current_line: LineNumber::new_universal(2),
            },
        );
    }

    #[test]
    fn it_can_process_none_ends() {
        let end_condition = EndCondition::default();

        for input in [&b"<%C%>"[..], &b"<%C%>after"[..]] {
            test(
                input,
                end_condition.clone(),
                0,
                b"",
                End::None,
                MockCursorContext {
                    cursor: 5,
                    current_line: LineNumber::new_universal(1),
                },
            );
        }
        test(
            b"foo<%C%>",
            end_condition.clone(),
            0,
            b"foo",
            End::None,
            MockCursorContext {
                cursor: 8,
                current_line: LineNumber::new_universal(1),
            },
        );
        test(
            b"foo   <%C%>",
            end_condition.clone(),
            0,
            b"foo   ",
            End::None,
            MockCursorContext {
                cursor: 11,
                current_line: LineNumber::new_universal(1),
            },
        );
        test(
            b"foo<%C\nline 2%>",
            end_condition.clone(),
            0,
            b"foo",
            End::None,
            MockCursorContext {
                cursor: 15,
                current_line: LineNumber::new_universal(2),
            },
        );
    }

    #[test]
    fn it_cannot_process_atx_closing_ends_if_not_enabled() {
        let end_condition = EndCondition::default();

        test(
            b"foo ==",
            end_condition.clone(),
            0,
            b"foo ==",
            End::Eof,
            MockCursorContext {
                cursor: 6,
                current_line: LineNumber::new_universal(1),
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
            ..Default::default()
        };

        test(
            b"foo ==",
            end_condition.clone(),
            0,
            b"foo",
            End::Eof,
            MockCursorContext {
                cursor: 6,
                current_line: LineNumber::new_universal(1),
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
                current_line: LineNumber::new_universal(1),
            },
        );
        test(
            b"foo ==\n",
            end_condition.clone(),
            0,
            b"foo",
            NewLine {
                line_after: LineNumber::new_universal(2),
            }
            .into(),
            MockCursorContext {
                cursor: 7,
                current_line: LineNumber::new_universal(2),
            },
        );
        test(
            b"foo ==   \n",
            end_condition.clone(),
            0,
            b"foo",
            NewLine {
                line_after: LineNumber::new_universal(2),
            }
            .into(),
            MockCursorContext {
                cursor: 10,
                current_line: LineNumber::new_universal(2),
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
                current_line: LineNumber::new_universal(1),
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
                current_line: LineNumber::new_universal(1),
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
                current_line: LineNumber::new_universal(1),
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
                current_line: LineNumber::new_universal(1),
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
            ..Default::default()
        };

        test(
            b"foo ==<`VE`>",
            end_condition.clone(),
            0,
            b"foo ==",
            VerbatimEscaping {
                content: 8..10,
                is_closed_forcedly: false,
                line_after: LineNumber::new_universal(1),
            }
            .into(),
            MockCursorContext {
                cursor: 12,
                current_line: LineNumber::new_universal(1),
            },
        );
    }

    #[test]
    fn it_cannot_process_table_related_ends_if_not_enabled() {
        let end_condition = EndCondition::default();

        test(
            b"|}",
            end_condition.clone(),
            0,
            b"|}",
            End::Eof,
            MockCursorContext {
                cursor: 2,
                current_line: LineNumber::new_universal(1),
            },
        );
    }

    #[test]
    fn it_can_process_table_related_ends_not_concerning_captions() {
        let end_condition = EndCondition {
            on_table_related: Some(TableRelated {
                is_caption_applicable: false,
            }),
            ..Default::default()
        };

        test(
            b"|}",
            end_condition.clone(),
            0,
            b"",
            table::TableRelatedEnd::Closing.into(),
            MockCursorContext {
                cursor: 2,
                current_line: LineNumber::new_universal(1),
            },
        );
        test(
            b"foo|}",
            end_condition.clone(),
            0,
            b"foo",
            table::TableRelatedEnd::Closing.into(),
            MockCursorContext {
                cursor: 5,
                current_line: LineNumber::new_universal(1),
            },
        );
        test(
            b"foo |}",
            end_condition.clone(),
            0,
            b"foo",
            table::TableRelatedEnd::Closing.into(),
            MockCursorContext {
                cursor: 6,
                current_line: LineNumber::new_universal(1),
            },
        );
        test(
            b"|-",
            end_condition.clone(),
            0,
            b"",
            table::TableRelatedEnd::RowIndicator.into(),
            MockCursorContext {
                cursor: 2,
                current_line: LineNumber::new_universal(1),
            },
        );
        test(
            b"!!",
            end_condition.clone(),
            0,
            b"",
            table::TableRelatedEnd::HeaderCellIndicator.into(),
            MockCursorContext {
                cursor: 2,
                current_line: LineNumber::new_universal(1),
            },
        );
        test(
            b"||",
            end_condition.clone(),
            0,
            b"",
            End::DoublePipes,
            MockCursorContext {
                cursor: 2,
                current_line: LineNumber::new_universal(1),
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
                current_line: LineNumber::new_universal(1),
            },
        );
    }

    #[test]
    fn it_can_process_table_caption_ends_if_applicable() {
        let end_condition = EndCondition {
            on_table_related: Some(TableRelated {
                is_caption_applicable: true,
            }),
            ..Default::default()
        };

        test(
            b"|+",
            end_condition.clone(),
            0,
            b"",
            table::TableRelatedEnd::CaptionIndicator.into(),
            MockCursorContext {
                cursor: 2,
                current_line: LineNumber::new_universal(1),
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
            ..Default::default()
        };

        test(
            b"foo ==|}",
            end_condition.clone(),
            0,
            b"foo",
            table::TableRelatedEnd::Closing.into(),
            MockCursorContext {
                cursor: 8,
                current_line: LineNumber::new_universal(1),
            },
        );
        test(
            b"foo == |}",
            end_condition.clone(),
            0,
            b"foo",
            table::TableRelatedEnd::Closing.into(),
            MockCursorContext {
                cursor: 9,
                current_line: LineNumber::new_universal(1),
            },
        );
        test(
            b"foo ==|-",
            end_condition.clone(),
            0,
            b"foo",
            table::TableRelatedEnd::RowIndicator.into(),
            MockCursorContext {
                cursor: 8,
                current_line: LineNumber::new_universal(1),
            },
        );
    }

    #[test]
    fn it_cannot_process_description_definition_opening_ends_if_not_enabled() {
        let end_condition = EndCondition::default();

        test(
            b"foo ::",
            end_condition.clone(),
            0,
            b"foo ::",
            End::Eof,
            MockCursorContext {
                cursor: 6,
                current_line: LineNumber::new_universal(1),
            },
        );
    }

    #[test]
    fn it_can_process_description_definition_opening_ends() {
        let end_condition = EndCondition {
            on_description_definition_opening: true,
            ..Default::default()
        };

        test(
            b"foo ::",
            end_condition.clone(),
            0,
            b"foo",
            End::DescriptionDefinitionOpening,
            MockCursorContext {
                cursor: 6,
                current_line: LineNumber::new_universal(1),
            },
        );
    }

    #[test]
    fn it_can_match_call_name_ends() {
        let end_condition = EndCondition {
            matching: Some(Matching::CallName),
            ..Default::default()
        };

        for suffix in [" ", "\t", "\r", "\n"] {
            let mut delta = 0;
            if suffix.as_bytes().first().is_some_and(|c| is_whitespace!(c)) {
                delta += 1;
            }

            test(
                format!("foo{}", suffix).as_bytes(),
                end_condition.clone(),
                0,
                b"",
                End::MatchedCallName {
                    is_extension: false,
                    range: 0..3,
                    extra_matched: MatchedCallNameExtraMatched::None,
                },
                MockCursorContext {
                    cursor: 3 + delta,
                    current_line: LineNumber::new_universal(1),
                },
            );
            test(
                format!("#foo{}", suffix).as_bytes(),
                end_condition.clone(),
                0,
                b"",
                End::MatchedCallName {
                    is_extension: true,
                    range: 1..4,
                    extra_matched: MatchedCallNameExtraMatched::None,
                },
                MockCursorContext {
                    cursor: 4 + delta,
                    current_line: LineNumber::new_universal(1),
                },
            );
        }

        test(
            b"foo||",
            end_condition.clone(),
            0,
            b"",
            End::MatchedCallName {
                is_extension: false,
                range: 0..3,
                extra_matched: MatchedCallNameExtraMatched::ArgumentIndicator,
            },
            MockCursorContext {
                cursor: 5,
                current_line: LineNumber::new_universal(1),
            },
        );
        test(
            b"foo ||",
            end_condition.clone(),
            0,
            b"",
            End::MatchedCallName {
                is_extension: false,
                range: 0..3,
                extra_matched: MatchedCallNameExtraMatched::ArgumentIndicator,
            },
            MockCursorContext {
                cursor: 6,
                current_line: LineNumber::new_universal(1),
            },
        );
        test(
            b"#foo||",
            end_condition.clone(),
            0,
            b"",
            End::MatchedCallName {
                is_extension: true,
                range: 1..4,
                extra_matched: MatchedCallNameExtraMatched::ArgumentIndicator,
            },
            MockCursorContext {
                cursor: 6,
                current_line: LineNumber::new_universal(1),
            },
        );
        test(
            b"#foo}}",
            end_condition.clone(),
            0,
            b"",
            End::MatchedCallName {
                is_extension: true,
                range: 1..4,
                extra_matched: MatchedCallNameExtraMatched::CallClosing,
            },
            MockCursorContext {
                cursor: 6,
                current_line: LineNumber::new_universal(1),
            },
        );

        for input in ["foo#", "foo|", "foo}", "foo*"] {
            test(
                input.as_bytes(),
                end_condition.clone(),
                0,
                b"foo",
                End::Mismatched,
                MockCursorContext {
                    cursor: 3,
                    current_line: LineNumber::new_universal(1),
                },
            );
        }
        test(
            b"foo |",
            end_condition.clone(),
            0,
            b"foo ",
            End::Mismatched,
            MockCursorContext {
                cursor: 4,
                current_line: LineNumber::new_universal(1),
            },
        );

        for input in ["#", "*", "#foo|", "#foo}", "#foo#"] {
            test(
                input.as_bytes(),
                end_condition.clone(),
                0,
                b"",
                End::Mismatched,
                MockCursorContext {
                    cursor: 0,
                    current_line: LineNumber::new_universal(1),
                },
            );
        }

        test(
            b"|",
            end_condition.clone(),
            0,
            b"",
            End::Mismatched,
            MockCursorContext {
                cursor: 0,
                current_line: LineNumber::new_universal(1),
            },
        );

        test(
            b"<`VE`>",
            end_condition.clone(),
            0,
            b"",
            VerbatimEscaping {
                content: 2..4,
                is_closed_forcedly: false,
                line_after: LineNumber::new_universal(1),
            }
            .into(),
            MockCursorContext {
                cursor: 6,
                current_line: LineNumber::new_universal(1),
            },
        );

        test(
            b"#<`VE`>",
            end_condition.clone(),
            0,
            b"",
            End::MatchedCallName {
                is_extension: true,
                range: 3..5,
                extra_matched: MatchedCallNameExtraMatched::None,
            },
            MockCursorContext {
                cursor: 7,
                current_line: LineNumber::new_universal(1),
            },
        );

        test(
            b"#<%C%>",
            end_condition.clone(),
            0,
            b"",
            End::Mismatched,
            MockCursorContext {
                cursor: 0,
                current_line: LineNumber::new_universal(1),
            },
        );
    }

    #[test]
    fn it_can_match_argument_indicator_ends() {
        let end_condition = EndCondition {
            matching: Some(Matching::CallArgumentIndicator),
            ..Default::default()
        };

        test(
            b"||",
            end_condition.clone(),
            0,
            b"",
            End::MatchedCallArgumentIndicator,
            MockCursorContext {
                cursor: 2,
                current_line: LineNumber::new_universal(1),
            },
        );
        test(
            b"}}",
            end_condition.clone(),
            0,
            b"",
            End::MatchedCallClosing,
            MockCursorContext {
                cursor: 2,
                current_line: LineNumber::new_universal(1),
            },
        );

        for input in ["foo||", "|", "?", "}"] {
            test(
                input.as_bytes(),
                end_condition.clone(),
                0,
                b"",
                End::Mismatched,
                MockCursorContext {
                    cursor: 0,
                    current_line: LineNumber::new_universal(1),
                },
            );
        }
    }

    #[test]
    fn it_can_match_argument_name_ends() {
        let end_condition = EndCondition {
            matching: Some(Matching::CallArgumentName),
            ..Default::default()
        };

        test(
            b"foo",
            end_condition.clone(),
            0,
            b"",
            End::MatchedArgumentName {
                is_verbatim: false,
                range: 0..3,
                has_matched_equal_sign: false,
            },
            MockCursorContext {
                cursor: 3,
                current_line: LineNumber::new_universal(1),
            },
        );

        test(
            b"foo=",
            end_condition.clone(),
            0,
            b"",
            End::MatchedArgumentName {
                is_verbatim: false,
                range: 0..3,
                has_matched_equal_sign: true,
            },
            MockCursorContext {
                cursor: 4,
                current_line: LineNumber::new_universal(1),
            },
        );

        test(
            b"=",
            end_condition.clone(),
            0,
            b"",
            End::Mismatched,
            MockCursorContext {
                cursor: 0,
                current_line: LineNumber::new_universal(1),
            },
        );
        test(
            b"foo*",
            end_condition.clone(),
            0,
            b"foo",
            End::Mismatched,
            MockCursorContext {
                cursor: 3,
                current_line: LineNumber::new_universal(1),
            },
        );
    }

    #[test]
    fn it_can_match_equal_sign_ends() {
        let end_condition = EndCondition {
            matching: Some(Matching::EqualSign),
            ..Default::default()
        };

        test(
            b"=",
            end_condition.clone(),
            0,
            b"",
            End::Matched,
            MockCursorContext {
                cursor: 1,
                current_line: LineNumber::new_universal(1),
            },
        );

        test(
            b"foo",
            end_condition.clone(),
            0,
            b"",
            End::Mismatched,
            MockCursorContext {
                cursor: 0,
                current_line: LineNumber::new_universal(1),
            },
        );
    }
}
