use indoc::indoc;

use rotext_core::EventType;

use crate::suites::block::support::{case, GroupedCases};

pub fn groups_paragraph() -> Vec<GroupedCases> {
    vec![
        GroupedCases {
            group: "段落",
            cases: vec![
                case!(
                    vec!["a", "␠a"],
                    vec![
                        (EventType::EnterParagraph, None),
                        (EventType::__Unparsed, Some("a")),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec!["a␠"],
                    vec![
                        (EventType::EnterParagraph, None),
                        (EventType::__Unparsed, Some("a")),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![
                        indoc! {"
                        a
                        b"},
                        indoc! {"
                        a
                        ␠b"},
                    ],
                    vec![
                        (EventType::EnterParagraph, None),
                        (EventType::__Unparsed, Some("a")),
                        (EventType::NewLine, None),
                        (EventType::__Unparsed, Some("b")),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![
                        indoc! {"
                        a

                        b"},
                        indoc! {"
                        a

                        ␠b"},
                    ],
                    vec![
                        (EventType::EnterParagraph, None),
                        (EventType::__Unparsed, Some("a")),
                        (EventType::ExitBlock, None),
                        (EventType::EnterParagraph, None),
                        (EventType::__Unparsed, Some("b")),
                        (EventType::ExitBlock, None),
                    ]
                ),
            ],
        },
        GroupedCases {
            group: "段落>段落与全局阶段语法的互动>逐字转义",
            cases: vec![
                case!(
                    vec!["a<`c`>"],
                    vec![
                        (EventType::EnterParagraph, None),
                        (EventType::__Unparsed, Some("a")),
                        (EventType::VerbatimEscaping, Some("c")),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec!["<`c`>b"],
                    vec![
                        (EventType::EnterParagraph, None),
                        (EventType::VerbatimEscaping, Some("c")),
                        (EventType::__Unparsed, Some("b")),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec!["<`c`>␣b"],
                    vec![
                        (EventType::EnterParagraph, None),
                        (EventType::VerbatimEscaping, Some("c")),
                        (EventType::__Unparsed, Some(" b")),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec!["a<`c`>b"],
                    vec![
                        (EventType::EnterParagraph, None),
                        (EventType::__Unparsed, Some("a")),
                        (EventType::VerbatimEscaping, Some("c")),
                        (EventType::__Unparsed, Some("b")),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![
                        indoc! {"
                        a
                        <`c`>"},
                        indoc! {"
                        a
                        ␠<`c`>"},
                    ],
                    vec![
                        (EventType::EnterParagraph, None),
                        (EventType::__Unparsed, Some("a")),
                        (EventType::NewLine, None),
                        (EventType::VerbatimEscaping, Some("c")),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![
                        indoc! {"
                        <`c`>
                        b"},
                        indoc! {"
                        <`c`>
                        ␠b"},
                    ],
                    vec![
                        (EventType::EnterParagraph, None),
                        (EventType::VerbatimEscaping, Some("c")),
                        (EventType::NewLine, None),
                        (EventType::__Unparsed, Some("b")),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec!["<`c`><`d`>"],
                    vec![
                        (EventType::EnterParagraph, None),
                        (EventType::VerbatimEscaping, Some("c")),
                        (EventType::VerbatimEscaping, Some("d")),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec!["<`c`>␣<`d`>"],
                    vec![
                        (EventType::EnterParagraph, None),
                        (EventType::VerbatimEscaping, Some("c")),
                        (EventType::__Unparsed, Some(" ")),
                        (EventType::VerbatimEscaping, Some("d")),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec!["<`c`>␣b<`d`>"],
                    vec![
                        (EventType::EnterParagraph, None),
                        (EventType::VerbatimEscaping, Some("c")),
                        (EventType::__Unparsed, Some(" b")),
                        (EventType::VerbatimEscaping, Some("d")),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec!["<`c`>b␣<`d`>"],
                    vec![
                        (EventType::EnterParagraph, None),
                        (EventType::VerbatimEscaping, Some("c")),
                        (EventType::__Unparsed, Some("b ")),
                        (EventType::VerbatimEscaping, Some("d")),
                        (EventType::ExitBlock, None),
                    ]
                ),
            ],
        },
        GroupedCases {
            group: "段落>段落与全局阶段语法的互动>注释",
            cases: vec![
                case!(
                    vec!["a<%c%>"],
                    vec![
                        (EventType::EnterParagraph, None),
                        (EventType::__Unparsed, Some("a")),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec!["<%c%>b"],
                    vec![
                        (EventType::EnterParagraph, None),
                        (EventType::__Unparsed, Some("b")),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec!["<%c%>␠b"],
                    vec![
                        (EventType::EnterParagraph, None),
                        (EventType::__Unparsed, Some("b")),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec!["a<%c%>b"],
                    vec![
                        (EventType::EnterParagraph, None),
                        (EventType::__Unparsed, Some("a")),
                        (EventType::__Unparsed, Some("b")),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![
                        indoc! {"
                        a
                        <%c%>"},
                        indoc! {"
                        a
                        ␠<%c%>"},
                    ],
                    vec![
                        (EventType::EnterParagraph, None),
                        (EventType::__Unparsed, Some("a")),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![
                        indoc! {"
                        <%c%>
                        b"},
                        indoc! {"
                        <%c%>
                        ␠b"},
                    ],
                    vec![
                        (EventType::EnterParagraph, None),
                        (EventType::__Unparsed, Some("b")),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(vec!["<%c%><%d%>", "<%c%>␠<%d%>"], vec![]),
                case!(
                    vec!["<%c%>␠b<%d%>"],
                    vec![
                        (EventType::EnterParagraph, None),
                        (EventType::__Unparsed, Some("b")),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec!["<%c%>b␣<%d%>"],
                    vec![
                        (EventType::EnterParagraph, None),
                        (EventType::__Unparsed, Some("b ")),
                        (EventType::ExitBlock, None),
                    ]
                ),
            ],
        },
        GroupedCases {
            group: "段落>“继续段落” 的优先级高于 “开启其他块级语法” 的优先级",
            cases: vec![
                case!(
                    // 分割线
                    vec![indoc! {"
                        a
                        ---"},],
                    vec![
                        (EventType::EnterParagraph, None),
                        (EventType::__Unparsed, Some("a")),
                        (EventType::NewLine, None),
                        (EventType::__Unparsed, Some("---")),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    // 块引用
                    vec![indoc! {"
                        a
                        >␣b"},],
                    vec![
                        (EventType::EnterParagraph, None),
                        (EventType::__Unparsed, Some("a")),
                        (EventType::NewLine, None),
                        (EventType::__Unparsed, Some("> b")),
                        (EventType::ExitBlock, None),
                    ]
                ),
            ],
        },
    ]
}
