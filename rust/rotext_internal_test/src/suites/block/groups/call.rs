use indoc::indoc;

use rotext_core::EventType;

use crate::suites::block::support::{case, GroupedCases};

pub fn groups_call() -> Vec<GroupedCases> {
    vec![
        GroupedCases {
            group: "调用>不匹配",
            cases: vec![
                case!(
                    vec!["{{"],
                    vec![
                        (EventType::EnterParagraph, None),
                        (EventType::__Unparsed, Some("{{")),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec!["{{}}"],
                    vec![
                        (EventType::EnterParagraph, None),
                        (EventType::__Unparsed, Some("{{}}")),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec!["{{␣}}"],
                    vec![
                        (EventType::EnterParagraph, None),
                        (EventType::__Unparsed, Some("{{ }}")),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec!["{{#␣foo}}"],
                    vec![
                        (EventType::EnterParagraph, None),
                        (EventType::__Unparsed, Some("{{# foo}}")),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec!["{{foo|"],
                    vec![
                        (EventType::EnterParagraph, None),
                        (EventType::__Unparsed, Some("{{foo|")),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec!["{{foo*"],
                    vec![
                        (EventType::EnterParagraph, None),
                        (EventType::__Unparsed, Some("{{foo*")),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec!["{{foo\n|"],
                    vec![
                        (EventType::EnterParagraph, None),
                        (EventType::__Unparsed, Some("{{foo")),
                        (EventType::NewLine, None),
                        (EventType::__Unparsed, Some("|")),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec!["{{foo\n\n|"],
                    vec![
                        (EventType::EnterParagraph, None),
                        (EventType::__Unparsed, Some("{{foo")),
                        (EventType::ExitBlock, None),
                        (EventType::EnterParagraph, None),
                        (EventType::__Unparsed, Some("|")),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![indoc! {"
                            *␠{{foo
                            >␠|"},],
                    vec![
                        (EventType::EnterUnorderedList, None),
                        (EventType::EnterListItem, None),
                        (EventType::EnterParagraph, None),
                        (EventType::__Unparsed, Some("{{foo")),
                        (EventType::NewLine, None),
                        (EventType::__Unparsed, Some("|")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![indoc! {"
                            *␠{{foo
                            >
                            >␠|"},],
                    vec![
                        (EventType::EnterUnorderedList, None),
                        (EventType::EnterListItem, None),
                        (EventType::EnterParagraph, None),
                        (EventType::__Unparsed, Some("{{foo")),
                        (EventType::ExitBlock, None),
                        (EventType::EnterParagraph, None),
                        (EventType::__Unparsed, Some("|")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
            ],
        },
        GroupedCases {
            group: "调用>无参数",
            cases: vec![
                case!(
                    vec![
                        "{{foo}}",
                        "{{␠foo␠}}",
                        "{{\nfoo\n}}",
                        "{{\n\nfoo\n\n}}",
                        "{{␠\n␠foo␠\n␠}}",
                        "{{<%C%>foo<%C%>}}",
                        "{{␠<%C%>␠foo␠<%C%>␠}}",
                        "{{\n<%C%>\nfoo\n<%C%>\n}}",
                        "{{foo",
                        "{{\n\nfoo\n\n",
                    ],
                    vec![
                        (EventType::EnterCallOnTemplate, Some("foo")),
                        (EventType::ExitBlock, None)
                    ]
                ),
                case!(
                    vec!["{{foo␣bar}}",],
                    vec![
                        (EventType::EnterCallOnTemplate, Some("foo bar")),
                        (EventType::ExitBlock, None)
                    ]
                ),
                case!(
                    vec!["{{<`*`>}}", "{{␠<`*`>␠}}", "{{\n<`*`>\n}}",],
                    vec![
                        (EventType::EnterCallOnTemplate, Some("*")),
                        (EventType::ExitBlock, None)
                    ]
                ),
                case!(
                    vec!["{{#foo}}", "{{#foo}}",],
                    vec![
                        (EventType::EnterCallOnExtension, Some("foo")),
                        (EventType::ExitBlock, None)
                    ]
                ),
                case!(
                    vec!["{{#<`*`>}}",],
                    vec![
                        (EventType::EnterCallOnExtension, Some("*")),
                        (EventType::ExitBlock, None)
                    ]
                ),
                case!(
                    vec![
                        "*␠{{foo}}",
                        indoc! {"
                            *␠{{
                            >␠foo}}"},
                        indoc! {"
                            *␠{{
                            >
                            >␠foo
                            >"},
                    ],
                    vec![
                        (EventType::EnterUnorderedList, None),
                        (EventType::EnterListItem, None),
                        (EventType::EnterCallOnTemplate, Some("foo")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
            ],
        },
        GroupedCases {
            group: "调用>单个匿名参数",
            cases: vec![
                case!(
                    vec!["{{foo||}}", "{{foo␠||}}", "{{foo\n||}}", "{{foo||"],
                    vec![
                        (EventType::EnterCallOnTemplate, Some("foo")),
                        (EventType::IndicateCallNormalArgument, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec!["{{foo||bar}}", "{{foo||␠bar␠}}", "{{foo\n||\nbar\n}}"],
                    vec![
                        (EventType::EnterCallOnTemplate, Some("foo")),
                        (EventType::IndicateCallNormalArgument, None),
                        (EventType::EnterParagraph, None),
                        (EventType::__Unparsed, Some("bar")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec!["{{foo||=}}",],
                    vec![
                        (EventType::EnterCallOnTemplate, Some("foo")),
                        (EventType::IndicateCallNormalArgument, None),
                        (EventType::EnterParagraph, None),
                        (EventType::__Unparsed, Some("=")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec!["{{foo||{{bar}}}}"],
                    vec![
                        (EventType::EnterCallOnTemplate, Some("foo")),
                        (EventType::IndicateCallNormalArgument, None),
                        (EventType::EnterCallOnTemplate, Some("bar")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec!["{{foo||{{bar||}}}}"],
                    vec![
                        (EventType::EnterCallOnTemplate, Some("foo")),
                        (EventType::IndicateCallNormalArgument, None),
                        (EventType::EnterCallOnTemplate, Some("bar")),
                        (EventType::IndicateCallNormalArgument, None),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec!["{{foo||\n*␠bar}}"],
                    vec![
                        (EventType::EnterCallOnTemplate, Some("foo")),
                        (EventType::IndicateCallNormalArgument, None),
                        (EventType::EnterUnorderedList, None),
                        (EventType::EnterListItem, None),
                        (EventType::EnterParagraph, None),
                        (EventType::__Unparsed, Some("bar")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec!["{{foo||*␣bar}}"],
                    vec![
                        (EventType::EnterCallOnTemplate, Some("foo")),
                        (EventType::IndicateCallNormalArgument, None),
                        (EventType::EnterParagraph, None),
                        (EventType::__Unparsed, Some("* bar")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec!["{{foo||{|bar||baz|}}}"],
                    vec![
                        (EventType::EnterCallOnTemplate, Some("foo")),
                        (EventType::IndicateCallNormalArgument, None),
                        (EventType::EnterTable, None),
                        (EventType::EnterParagraph, None),
                        (EventType::__Unparsed, Some("bar")),
                        (EventType::ExitBlock, None),
                        (EventType::IndicateTableDataCell, None),
                        (EventType::EnterParagraph, None),
                        (EventType::__Unparsed, Some("baz")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec!["{{foo||<`bar`><`baz`>}}"],
                    vec![
                        (EventType::EnterCallOnTemplate, Some("foo")),
                        (EventType::IndicateCallNormalArgument, None),
                        (EventType::EnterParagraph, None),
                        (EventType::VerbatimEscaping, Some("bar")),
                        (EventType::VerbatimEscaping, Some("baz")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec!["{{foo||␠`}}"],
                    vec![
                        (EventType::EnterCallOnTemplate, Some("foo")),
                        (EventType::IndicateCallNormalArgument, None),
                        (EventType::EnterParagraph, None),
                        (EventType::__Unparsed, Some("`")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
            ],
        },
        GroupedCases {
            group: "调用>单个命名参数",
            cases: vec![
                case!(
                    vec![
                        "{{foo||bar=}}",
                        "{{foo||␠bar␠=␠}}",
                        "{{foo\n||\nbar\n=\n}}",
                        "{{foo||<%C%>bar<%C%>=}}",
                        "{{foo||␠<%C%>␠bar␠<%C%>␠=}}",
                        "{{foo||\n<%C%>\nbar\n<%C%>\n=}}",
                        "{{foo||bar="
                    ],
                    vec![
                        (EventType::EnterCallOnTemplate, Some("foo")),
                        (EventType::IndicateCallNormalArgument, Some("bar")),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![
                        "{{foo||bar=baz}}",
                        "{{foo||␠bar␠=␠baz␠}}",
                        "{{foo\n||\nbar\n=\nbaz\n}}",
                        "{{foo||bar=baz"
                    ],
                    vec![
                        (EventType::EnterCallOnTemplate, Some("foo")),
                        (EventType::IndicateCallNormalArgument, Some("bar")),
                        (EventType::EnterParagraph, None),
                        (EventType::__Unparsed, Some("baz")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec!["{{foo||bar␣baz=qux}}",],
                    vec![
                        (EventType::EnterCallOnTemplate, Some("foo")),
                        (EventType::IndicateCallNormalArgument, Some("bar baz")),
                        (EventType::EnterParagraph, None),
                        (EventType::__Unparsed, Some("qux")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec!["{{foo||<`*`>=}}",],
                    vec![
                        (EventType::EnterCallOnTemplate, Some("foo")),
                        (EventType::IndicateCallNormalArgument, Some("*")),
                        (EventType::ExitBlock, None),
                    ]
                ),
            ],
        },
        GroupedCases {
            group: "调用>单个逐字匿名参数",
            cases: vec![
                case!(
                    vec!["{{foo||`}}"],
                    vec![
                        (EventType::EnterCallOnTemplate, Some("foo")),
                        (EventType::IndicateCallVerbatimArgument, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec!["{{foo||`bar}}"],
                    vec![
                        (EventType::EnterCallOnTemplate, Some("foo")),
                        (EventType::IndicateCallVerbatimArgument, None),
                        (EventType::Text, Some("bar")),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec!["{{foo||`=}}",],
                    vec![
                        (EventType::EnterCallOnTemplate, Some("foo")),
                        (EventType::IndicateCallVerbatimArgument, None),
                        (EventType::Text, Some("=")),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec!["{{foo||`<`bar`><`baz`>}}"],
                    vec![
                        (EventType::EnterCallOnTemplate, Some("foo")),
                        (EventType::IndicateCallVerbatimArgument, None),
                        (EventType::VerbatimEscaping, Some("bar")),
                        (EventType::VerbatimEscaping, Some("baz")),
                        (EventType::ExitBlock, None),
                    ]
                ),
            ],
        },
        GroupedCases {
            group: "调用>单个逐字命名参数",
            cases: vec![
                case!(
                    vec![
                        "{{foo||`bar=}}",
                        "{{foo||␠`bar␠=}}",
                        "{{foo\n||\n`bar\n=}}",
                        "{{foo||<%C%>`bar<%C%>=}}",
                        "{{foo||␠<%C%>␠`bar␠<%C%>␠=}}",
                        "{{foo||\n<%C%>\n`bar\n<%C%>\n=}}",
                        "{{foo||`bar=",
                    ],
                    vec![
                        (EventType::EnterCallOnTemplate, Some("foo")),
                        (EventType::IndicateCallVerbatimArgument, Some("bar")),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec!["{{foo||`bar␣baz=}}",],
                    vec![
                        (EventType::EnterCallOnTemplate, Some("foo")),
                        (EventType::IndicateCallVerbatimArgument, Some("bar baz")),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec!["{{foo||`<`*`>=␣}}",],
                    vec![
                        (EventType::EnterCallOnTemplate, Some("foo")),
                        (EventType::IndicateCallVerbatimArgument, Some("*")),
                        (EventType::Text, Some(" ")),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec!["{{foo||`bar=\n}}",],
                    vec![
                        (EventType::EnterCallOnTemplate, Some("foo")),
                        (EventType::IndicateCallVerbatimArgument, Some("bar")),
                        (EventType::NewLine, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![indoc! {"
                            {{foo||`bar=1
                            2
                            
                            3}}"},],
                    vec![
                        (EventType::EnterCallOnTemplate, Some("foo")),
                        (EventType::IndicateCallVerbatimArgument, Some("bar")),
                        (EventType::Text, Some("1")),
                        (EventType::NewLine, None),
                        (EventType::Text, Some("2")),
                        (EventType::NewLine, None),
                        (EventType::NewLine, None),
                        (EventType::Text, Some("3")),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec!["*␠{{foo||`bar=1}}", "*␠{{foo||`bar=1",],
                    vec![
                        (EventType::EnterUnorderedList, None),
                        (EventType::EnterListItem, None),
                        (EventType::EnterCallOnTemplate, Some("foo")),
                        (EventType::IndicateCallVerbatimArgument, Some("bar")),
                        (EventType::Text, Some("1")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![indoc! {"
                            *␠{{foo||`bar=1
                            >␠␣2}}"},],
                    vec![
                        (EventType::EnterUnorderedList, None),
                        (EventType::EnterListItem, None),
                        (EventType::EnterCallOnTemplate, Some("foo")),
                        (EventType::IndicateCallVerbatimArgument, Some("bar")),
                        (EventType::Text, Some("1")),
                        (EventType::NewLine, None),
                        (EventType::Text, Some(" 2")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![
                        indoc! {"
                            *␠{{foo||`bar=1
                            >␠2
                            >
                            >␠3}}"},
                        indoc! {"
                            *␠{{foo||`bar=1
                            >␠2
                            >
                            >␠3"},
                    ],
                    vec![
                        (EventType::EnterUnorderedList, None),
                        (EventType::EnterListItem, None),
                        (EventType::EnterCallOnTemplate, Some("foo")),
                        (EventType::IndicateCallVerbatimArgument, Some("bar")),
                        (EventType::Text, Some("1")),
                        (EventType::NewLine, None),
                        (EventType::Text, Some("2")),
                        (EventType::NewLine, None),
                        (EventType::NewLine, None),
                        (EventType::Text, Some("3")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![indoc! {"
                            *␠{{foo||`bar=␣
                            >␠␣
                            >␠}}"},],
                    vec![
                        (EventType::EnterUnorderedList, None),
                        (EventType::EnterListItem, None),
                        (EventType::EnterCallOnTemplate, Some("foo")),
                        (EventType::IndicateCallVerbatimArgument, Some("bar")),
                        (EventType::Text, Some(" ")),
                        (EventType::NewLine, None),
                        (EventType::Text, Some(" ")),
                        (EventType::NewLine, None),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
            ],
        },
        GroupedCases {
            group: "调用>多个参数",
            cases: vec![
                case!(
                    vec!["{{foo||bar||baz␣k=baz␣v||qux||quux␣k=quux␣v}}",],
                    vec![
                        (EventType::EnterCallOnTemplate, Some("foo")),
                        (EventType::IndicateCallNormalArgument, None),
                        (EventType::EnterParagraph, None),
                        (EventType::__Unparsed, Some("bar")),
                        (EventType::ExitBlock, None),
                        (EventType::IndicateCallNormalArgument, Some("baz k")),
                        (EventType::EnterParagraph, None),
                        (EventType::__Unparsed, Some("baz v")),
                        (EventType::ExitBlock, None),
                        (EventType::IndicateCallNormalArgument, None),
                        (EventType::EnterParagraph, None),
                        (EventType::__Unparsed, Some("qux")),
                        (EventType::ExitBlock, None),
                        (EventType::IndicateCallNormalArgument, Some("quux k")),
                        (EventType::EnterParagraph, None),
                        (EventType::__Unparsed, Some("quux v")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![indoc! {"
                            {{foo||
                            *␠||
                            >␠||
                            }}"},],
                    vec![
                        (EventType::EnterCallOnTemplate, Some("foo")),
                        (EventType::IndicateCallNormalArgument, None),
                        (EventType::EnterUnorderedList, None),
                        (EventType::EnterListItem, None),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::IndicateCallNormalArgument, None),
                        (EventType::EnterBlockQuote, None),
                        (EventType::ExitBlock, None),
                        (EventType::IndicateCallNormalArgument, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![indoc! {"
                            *␠{{foo||
                            >␠*␠||
                            >␠>␠||
                            >␠}}"},],
                    vec![
                        (EventType::EnterUnorderedList, None),
                        (EventType::EnterListItem, None),
                        (EventType::EnterCallOnTemplate, Some("foo")),
                        (EventType::IndicateCallNormalArgument, None),
                        (EventType::EnterUnorderedList, None),
                        (EventType::EnterListItem, None),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::IndicateCallNormalArgument, None),
                        (EventType::EnterBlockQuote, None),
                        (EventType::ExitBlock, None),
                        (EventType::IndicateCallNormalArgument, None),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec!["{{foo||`bar=baz||qux}}",],
                    vec![
                        (EventType::EnterCallOnTemplate, Some("foo")),
                        (EventType::IndicateCallVerbatimArgument, Some("bar")),
                        (EventType::Text, Some("baz")),
                        (EventType::IndicateCallNormalArgument, None),
                        (EventType::EnterParagraph, None),
                        (EventType::__Unparsed, Some("qux")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![indoc! {"
                            {{foo||`bar=baz
                            ||qux}}"},],
                    vec![
                        (EventType::EnterCallOnTemplate, Some("foo")),
                        (EventType::IndicateCallVerbatimArgument, Some("bar")),
                        (EventType::Text, Some("baz")),
                        (EventType::NewLine, None),
                        (EventType::IndicateCallNormalArgument, None),
                        (EventType::EnterParagraph, None),
                        (EventType::__Unparsed, Some("qux")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![indoc! {"
                            {{foo||`bar=baz
                            ␣||qux}}"},],
                    vec![
                        (EventType::EnterCallOnTemplate, Some("foo")),
                        (EventType::IndicateCallVerbatimArgument, Some("bar")),
                        (EventType::Text, Some("baz")),
                        (EventType::NewLine, None),
                        (EventType::Text, Some(" ")),
                        (EventType::IndicateCallNormalArgument, None),
                        (EventType::EnterParagraph, None),
                        (EventType::__Unparsed, Some("qux")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![indoc! {"
                            {{foo||`bar=baz

                            ||qux}}"},],
                    vec![
                        (EventType::EnterCallOnTemplate, Some("foo")),
                        (EventType::IndicateCallVerbatimArgument, Some("bar")),
                        (EventType::Text, Some("baz")),
                        (EventType::NewLine, None),
                        (EventType::NewLine, None),
                        (EventType::IndicateCallNormalArgument, None),
                        (EventType::EnterParagraph, None),
                        (EventType::__Unparsed, Some("qux")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
            ],
        },
    ]
}
