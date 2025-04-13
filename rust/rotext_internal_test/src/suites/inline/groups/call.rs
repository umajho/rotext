use indoc::indoc;

use rotext_core::EventType;

use crate::suites::inline::support::{GroupedCases, case};

pub fn groups_call() -> Vec<GroupedCases> {
    vec![
        GroupedCases {
            group: "调用>不匹配",
            cases: vec![
                case!(vec!["[{"], vec![(EventType::Text, Some("[{")),]),
                case!(vec!["[{}]"], vec![(EventType::Text, Some("[{}]")),]),
                case!(vec!["[{␣}]"], vec![(EventType::Text, Some("[{ }]")),]),
                case!(vec!["[{\nfoo}]"], vec![
                    (EventType::Text, Some("[{")),
                    (EventType::NewLine, None),
                    (EventType::Text, Some("foo}]")),
                ]),
                case!(vec!["[{foo\n}]"], vec![
                    (EventType::Text, Some("[{foo")),
                    (EventType::NewLine, None),
                    (EventType::Text, Some("}]")),
                ]),
                case!(vec!["[{foo\n|"], vec![
                    (EventType::Text, Some("[{foo")),
                    (EventType::NewLine, None),
                    (EventType::Text, Some("|")),
                ]),
                case!(vec!["[{<%C%>foo<%C%>}]"], vec![
                    (EventType::Text, Some("[{")),
                    (EventType::Text, Some("foo")),
                    (EventType::Text, Some("}]")),
                ]),
                case!(vec!["[{foo"], vec![(EventType::Text, Some("[{foo")),]),
                case!(vec!["[{#␣foo}]"], vec![(
                    EventType::Text,
                    Some("[{# foo}]")
                ),]),
                case!(vec!["[{foo["], vec![(EventType::Text, Some("[{foo[")),]),
            ],
        },
        GroupedCases {
            group: "调用>无参数",
            cases: vec![
                case!(vec!["[{foo}]", "[{␠foo␠}]",], vec![
                    (EventType::EnterCallOnTemplate, Some("foo")),
                    (EventType::ExitInline, None)
                ]),
                case!(vec!["[{foo␣bar}]",], vec![
                    (EventType::EnterCallOnTemplate, Some("foo bar")),
                    (EventType::ExitInline, None)
                ]),
                case!(vec!["[{<`*`>}]", "[{␠<`*`>␠}]",], vec![
                    (EventType::EnterCallOnTemplate, Some("*")),
                    (EventType::ExitInline, None)
                ]),
                case!(vec!["[{#foo}]", "[{␠#foo␠}]",], vec![
                    (EventType::EnterCallOnExtension, Some("foo")),
                    (EventType::ExitInline, None)
                ]),
                case!(vec!["[{#<`*`>}]",], vec![
                    (EventType::EnterCallOnExtension, Some("*")),
                    (EventType::ExitInline, None)
                ]),
            ],
        },
        GroupedCases {
            group: "调用>单个匿名参数",
            cases: vec![
                case!(vec!["[{foo|}]", "[{foo␠|}]", "[{foo|",], vec![
                    (EventType::EnterCallOnTemplate, Some("foo")),
                    (EventType::IndicateCallNormalArgument, None),
                    (EventType::ExitInline, None),
                ]),
                case!(vec!["[{foo|bar}]",], vec![
                    (EventType::EnterCallOnTemplate, Some("foo")),
                    (EventType::IndicateCallNormalArgument, None),
                    (EventType::Text, Some("bar")),
                    (EventType::ExitInline, None),
                ]),
                case!(vec!["[{foo|␣bar␣}]",], vec![
                    (EventType::EnterCallOnTemplate, Some("foo")),
                    (EventType::IndicateCallNormalArgument, None),
                    (EventType::Text, Some(" bar ")),
                    (EventType::ExitInline, None),
                ]),
                case!(vec!["[{foo|=}]",], vec![
                    (EventType::EnterCallOnTemplate, Some("foo")),
                    (EventType::IndicateCallNormalArgument, None),
                    (EventType::Text, Some("=")),
                    (EventType::ExitInline, None),
                ]),
                case!(vec!["[{foo|[{bar}]}]",], vec![
                    (EventType::EnterCallOnTemplate, Some("foo")),
                    (EventType::IndicateCallNormalArgument, None),
                    (EventType::EnterCallOnTemplate, Some("bar")),
                    (EventType::ExitInline, None),
                    (EventType::ExitInline, None),
                ]),
                case!(vec!["[{foo|[{bar|}]}]",], vec![
                    (EventType::EnterCallOnTemplate, Some("foo")),
                    (EventType::IndicateCallNormalArgument, None),
                    (EventType::EnterCallOnTemplate, Some("bar")),
                    (EventType::IndicateCallNormalArgument, None),
                    (EventType::ExitInline, None),
                    (EventType::ExitInline, None),
                ]),
                case!(vec!["[{foo|\n[{bar|}]}]",], vec![
                    (EventType::EnterCallOnTemplate, Some("foo")),
                    (EventType::IndicateCallNormalArgument, None),
                    (EventType::NewLine, None),
                    (EventType::EnterCallOnTemplate, Some("bar")),
                    (EventType::IndicateCallNormalArgument, None),
                    (EventType::ExitInline, None),
                    (EventType::ExitInline, None),
                ]),
                case!(vec!["[{foo|[*bar*]}]",], vec![
                    (EventType::EnterCallOnTemplate, Some("foo")),
                    (EventType::IndicateCallNormalArgument, None),
                    (EventType::EnterStrong, None),
                    (EventType::Text, Some("bar")),
                    (EventType::ExitInline, None),
                    (EventType::ExitInline, None),
                ]),
                case!(vec!["[{foo|\n[*bar*]}]",], vec![
                    (EventType::EnterCallOnTemplate, Some("foo")),
                    (EventType::IndicateCallNormalArgument, None),
                    (EventType::NewLine, None),
                    (EventType::EnterStrong, None),
                    (EventType::Text, Some("bar")),
                    (EventType::ExitInline, None),
                    (EventType::ExitInline, None),
                ]),
                case!(vec!["[{foo|<`bar`><`baz`>}]",], vec![
                    (EventType::EnterCallOnTemplate, Some("foo")),
                    (EventType::IndicateCallNormalArgument, None),
                    (EventType::VerbatimEscaping, Some("bar")),
                    (EventType::VerbatimEscaping, Some("baz")),
                    (EventType::ExitInline, None),
                ]),
                case!(vec!["[{foo|␣`}]",], vec![
                    (EventType::EnterCallOnTemplate, Some("foo")),
                    (EventType::IndicateCallNormalArgument, None),
                    (EventType::Text, Some(" `")),
                    (EventType::ExitInline, None),
                ]),
            ],
        },
        GroupedCases {
            group: "调用>单个命名参数",
            cases: vec![
                case!(vec!["[{foo|bar=}]", "[{foo|␠bar␠=}]", "[{foo|bar=",], vec![
                    (EventType::EnterCallOnTemplate, Some("foo")),
                    (EventType::IndicateCallNormalArgument, Some("bar")),
                    (EventType::ExitInline, None),
                ]),
                case!(vec!["[{foo|bar=␣}]",], vec![
                    (EventType::EnterCallOnTemplate, Some("foo")),
                    (EventType::IndicateCallNormalArgument, Some("bar")),
                    (EventType::Text, Some(" ")),
                    (EventType::ExitInline, None),
                ]),
                case!(
                    vec!["[{foo|bar=baz}]", "[{foo|␠bar␠=baz}]", "[{foo|bar=baz",],
                    vec![
                        (EventType::EnterCallOnTemplate, Some("foo")),
                        (EventType::IndicateCallNormalArgument, Some("bar")),
                        (EventType::Text, Some("baz")),
                        (EventType::ExitInline, None),
                    ]
                ),
                case!(vec!["[{foo|bar␣baz=qux}]",], vec![
                    (EventType::EnterCallOnTemplate, Some("foo")),
                    (EventType::IndicateCallNormalArgument, Some("bar baz")),
                    (EventType::Text, Some("qux")),
                    (EventType::ExitInline, None),
                ]),
                case!(vec!["[{foo|<`*`>=}]",], vec![
                    (EventType::EnterCallOnTemplate, Some("foo")),
                    (EventType::IndicateCallNormalArgument, Some("*")),
                    (EventType::ExitInline, None),
                ]),
            ],
        },
        GroupedCases {
            group: "调用>单个逐字匿名参数",
            cases: vec![
                case!(vec!["[{foo|`}]"], vec![
                    (EventType::EnterCallOnTemplate, Some("foo")),
                    (EventType::IndicateCallVerbatimArgument, None),
                    (EventType::ExitInline, None),
                ]),
                case!(vec!["[{foo|`bar}]"], vec![
                    (EventType::EnterCallOnTemplate, Some("foo")),
                    (EventType::IndicateCallVerbatimArgument, None),
                    (EventType::Text, Some("bar")),
                    (EventType::ExitInline, None),
                ]),
                case!(vec!["[{foo|`=}]",], vec![
                    (EventType::EnterCallOnTemplate, Some("foo")),
                    (EventType::IndicateCallVerbatimArgument, None),
                    (EventType::Text, Some("=")),
                    (EventType::ExitInline, None),
                ]),
                case!(vec!["[{foo|`<`bar`><`baz`>}]"], vec![
                    (EventType::EnterCallOnTemplate, Some("foo")),
                    (EventType::IndicateCallVerbatimArgument, None),
                    (EventType::VerbatimEscaping, Some("bar")),
                    (EventType::VerbatimEscaping, Some("baz")),
                    (EventType::ExitInline, None),
                ]),
                case!(vec!["[{foo|`[{bar}]}]"], vec![
                    (EventType::EnterCallOnTemplate, Some("foo")),
                    (EventType::IndicateCallVerbatimArgument, None),
                    (EventType::Text, Some("[{bar")),
                    (EventType::ExitInline, None),
                    (EventType::Text, Some("}]")),
                ]),
                case!(vec!["[{foo|`[*bar*]}]"], vec![
                    (EventType::EnterCallOnTemplate, Some("foo")),
                    (EventType::IndicateCallVerbatimArgument, None),
                    (EventType::Text, Some("[*bar*]")),
                    (EventType::ExitInline, None),
                ]),
            ],
        },
        GroupedCases {
            group: "调用>单个逐字命名参数",
            cases: vec![
                case!(
                    vec!["[{foo|`bar=}]", "[{foo|␠`bar␠=}]", "[{foo|`bar=",],
                    vec![
                        (EventType::EnterCallOnTemplate, Some("foo")),
                        (EventType::IndicateCallVerbatimArgument, Some("bar")),
                        (EventType::ExitInline, None),
                    ]
                ),
                case!(vec!["[{foo|`bar␣baz=}]",], vec![
                    (EventType::EnterCallOnTemplate, Some("foo")),
                    (EventType::IndicateCallVerbatimArgument, Some("bar baz")),
                    (EventType::ExitInline, None),
                ]),
                case!(vec!["[{foo|`<`*`>=␣}]",], vec![
                    (EventType::EnterCallOnTemplate, Some("foo")),
                    (EventType::IndicateCallVerbatimArgument, Some("*")),
                    (EventType::Text, Some(" ")),
                    (EventType::ExitInline, None),
                ]),
                case!(vec!["[{foo|`bar=\n}]",], vec![
                    (EventType::EnterCallOnTemplate, Some("foo")),
                    (EventType::IndicateCallVerbatimArgument, Some("bar")),
                    (EventType::NewLine, None),
                    (EventType::ExitInline, None),
                ]),
                case!(
                    vec![indoc! {"
                            [{foo|`bar=1
                            2
                            3}]"},],
                    vec![
                        (EventType::EnterCallOnTemplate, Some("foo")),
                        (EventType::IndicateCallVerbatimArgument, Some("bar")),
                        (EventType::Text, Some("1")),
                        (EventType::NewLine, None),
                        (EventType::Text, Some("2")),
                        (EventType::NewLine, None),
                        (EventType::Text, Some("3")),
                        (EventType::ExitInline, None),
                    ]
                ),
                case!(vec!["[{foo|`bar=[*baz*]}]"], vec![
                    (EventType::EnterCallOnTemplate, Some("foo")),
                    (EventType::IndicateCallVerbatimArgument, Some("bar")),
                    (EventType::Text, Some("[*baz*]")),
                    (EventType::ExitInline, None),
                ]),
            ],
        },
        GroupedCases {
            group: "调用>多个参数",
            cases: vec![
                case!(vec!["[{foo|bar|baz␣k=baz␣v|qux|quux␣k=quux␣v}]",], vec![
                    (EventType::EnterCallOnTemplate, Some("foo")),
                    (EventType::IndicateCallNormalArgument, None),
                    (EventType::Text, Some("bar")),
                    (EventType::IndicateCallNormalArgument, Some("baz k")),
                    (EventType::Text, Some("baz v")),
                    (EventType::IndicateCallNormalArgument, None),
                    (EventType::Text, Some("qux")),
                    (EventType::IndicateCallNormalArgument, Some("quux k")),
                    (EventType::Text, Some("quux v")),
                    (EventType::ExitInline, None),
                ]),
                case!(
                    vec![indoc! {"
                            [{foo|
                            bar␣|
                            baz␣|
                            }]"},],
                    vec![
                        (EventType::EnterCallOnTemplate, Some("foo")),
                        (EventType::IndicateCallNormalArgument, None),
                        (EventType::NewLine, None),
                        (EventType::Text, Some("bar ")),
                        (EventType::IndicateCallNormalArgument, None),
                        (EventType::NewLine, None),
                        (EventType::Text, Some("baz ")),
                        (EventType::IndicateCallNormalArgument, None),
                        (EventType::NewLine, None),
                        (EventType::ExitInline, None),
                    ]
                ),
                case!(vec!["[{foo|`bar=baz|qux}]",], vec![
                    (EventType::EnterCallOnTemplate, Some("foo")),
                    (EventType::IndicateCallVerbatimArgument, Some("bar")),
                    (EventType::Text, Some("baz")),
                    (EventType::IndicateCallNormalArgument, None),
                    (EventType::Text, Some("qux")),
                    (EventType::ExitInline, None),
                ]),
                case!(
                    vec![indoc! {"
                            [{foo|`bar=baz
                            |qux}]"},],
                    vec![
                        (EventType::EnterCallOnTemplate, Some("foo")),
                        (EventType::IndicateCallVerbatimArgument, Some("bar")),
                        (EventType::Text, Some("baz")),
                        (EventType::NewLine, None),
                        (EventType::IndicateCallNormalArgument, None),
                        (EventType::Text, Some("qux")),
                        (EventType::ExitInline, None),
                    ]
                ),
                case!(
                    vec![indoc! {"
                            [{foo|`bar=baz
                            ␣quux|qux}]"},],
                    vec![
                        (EventType::EnterCallOnTemplate, Some("foo")),
                        (EventType::IndicateCallVerbatimArgument, Some("bar")),
                        (EventType::Text, Some("baz")),
                        (EventType::NewLine, None),
                        (EventType::Text, Some("quux")), // NOTE: 行内阶段一行开头的空白会被削去。
                        (EventType::IndicateCallNormalArgument, None),
                        (EventType::Text, Some("qux")),
                        (EventType::ExitInline, None),
                    ]
                ),
            ],
        },
    ]
}
