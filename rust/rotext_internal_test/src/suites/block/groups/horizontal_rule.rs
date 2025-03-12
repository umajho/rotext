use indoc::indoc;

use rotext_core::EventType;

use crate::suites::block::support::{case, GroupedCases};

pub fn groups_horizontal_rule() -> Vec<GroupedCases> {
    vec![
        GroupedCases {
            group: "分割线",
            cases: vec![
                case!(
                    vec!["---", "----", "␠---"],
                    vec![(EventType::ThematicBreak, None)]
                ),
                case!(
                    vec!["--"],
                    vec![
                        (EventType::EnterParagraph, None),
                        (EventType::__Unparsed, Some("--")),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![
                        indoc! {"
                        ---
                        ---"},
                        "---␠---",
                    ],
                    vec![
                        (EventType::ThematicBreak, None),
                        (EventType::ThematicBreak, None),
                    ]
                ),
                case!(
                    vec![
                        indoc! {"
                        ---
                        a"},
                        "---a",
                        "---␠a",
                    ],
                    vec![
                        (EventType::ThematicBreak, None),
                        (EventType::EnterParagraph, None),
                        (EventType::__Unparsed, Some("a")),
                        (EventType::ExitBlock, None),
                    ]
                ),
            ],
        },
        GroupedCases {
            group: "分割线>分割线与全局阶段语法的互动>逐字转义",
            cases: vec![case!(
                vec![
                    indoc! {"
                        ---
                        <`a`>"},
                    "---<`a`>",
                    "---␠<`a`>",
                ],
                vec![
                    (EventType::ThematicBreak, None),
                    (EventType::EnterParagraph, None),
                    (EventType::VerbatimEscaping, Some("a")),
                    (EventType::ExitBlock, None),
                ]
            )],
        },
        GroupedCases {
            group: "分割线>分割线与全局阶段语法的互动>注释",
            cases: vec![case!(
                vec![
                    indoc! {"
                        ---
                        <%a%>"},
                    "---<%a%>",
                    "---␠<%a%>",
                ],
                vec![(EventType::ThematicBreak, None),]
            )],
        },
    ]
}
