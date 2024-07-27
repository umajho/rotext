use indoc::indoc;

use crate::{
    block::tests::{utils::case, GroupedCases},
    events::EventType,
};

pub fn groups_paragraph() -> Vec<GroupedCases> {
    vec![
        GroupedCases {
            group: "段落",
            cases: vec![
                case!(
                    vec!["a", " a"],
                    vec![
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("a")),
                        (EventType::Exit, None)
                    ]
                ),
                case!(
                    vec!["a "],
                    vec![
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("a ")),
                        (EventType::Exit, None)
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
                        (EventType::Unparsed, Some("a")),
                        (EventType::NewLine, None),
                        (EventType::Unparsed, Some("b")),
                        (EventType::Exit, None)
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
                        (EventType::Unparsed, Some("a")),
                        (EventType::Exit, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("b")),
                        (EventType::Exit, None),
                    ]
                ),
            ],
        },
        GroupedCases {
            group: "段落>段落与全局阶段语法的互动",
            cases: vec![
                case!(
                    vec!["a<`c`>"],
                    vec![
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("a")),
                        (EventType::VerbatimEscaping, Some("c")),
                        (EventType::Exit, None),
                    ]
                ),
                case!(
                    vec!["<`c`>b"],
                    vec![
                        (EventType::EnterParagraph, None),
                        (EventType::VerbatimEscaping, Some("c")),
                        (EventType::Unparsed, Some("b")),
                        (EventType::Exit, None),
                    ]
                ),
                case!(
                    vec!["a<`c`>b"],
                    vec![
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("a")),
                        (EventType::VerbatimEscaping, Some("c")),
                        (EventType::Unparsed, Some("b")),
                        (EventType::Exit, None),
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
                        (EventType::Unparsed, Some("a")),
                        (EventType::NewLine, None),
                        (EventType::VerbatimEscaping, Some("c")),
                        (EventType::Exit, None),
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
                        (EventType::Unparsed, Some("a")),
                        (EventType::NewLine, None),
                        (EventType::Unparsed, Some("---")),
                        (EventType::Exit, None),
                    ]
                ),
                case!(
                    // 块引用
                    vec![indoc! {"
                        a
                        > b"},],
                    vec![
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("a")),
                        (EventType::NewLine, None),
                        (EventType::Unparsed, Some("> b")),
                        (EventType::Exit, None),
                    ]
                ),
            ],
        },
    ]
}
