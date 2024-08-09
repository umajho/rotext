use indoc::indoc;

use crate::{
    events::EventType,
    test_suites::block::support::{case, GroupedCases},
};

pub fn groups_horizontal_rule() -> Vec<GroupedCases> {
    vec![
        GroupedCases {
            group: "分割线",
            cases: vec![
                case!(vec!["---", "----"], vec![(EventType::ThematicBreak, None)]),
                case!(
                    vec!["--"],
                    vec![
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("--")),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![
                        indoc! {"
                        ---
                        ---"},
                        "--- ---",
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
                        "--- a",
                    ],
                    vec![
                        (EventType::ThematicBreak, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("a")),
                        (EventType::ExitBlock, None),
                    ]
                ),
            ],
        },
        GroupedCases {
            group: "分割线>分割线与全局阶段语法的互动",
            cases: vec![case!(
                vec![
                    indoc! {"
                        ---
                        <`a`>"},
                    "---<`a`>",
                    "--- <`a`>",
                ],
                vec![
                    (EventType::ThematicBreak, None),
                    (EventType::EnterParagraph, None),
                    (EventType::VerbatimEscaping, Some("a")),
                    (EventType::ExitBlock, None),
                ]
            )],
        },
    ]
}
