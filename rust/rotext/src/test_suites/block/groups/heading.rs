use indoc::indoc;

use crate::{
    events::EventType,
    test_suites::block::support::{case, GroupedCases},
};

pub fn groups_heading() -> Vec<GroupedCases> {
    vec![
        GroupedCases {
            group: "标题",
            cases: vec![
                case!(
                    vec!["= a ="],
                    vec![
                        (EventType::EnterHeading1, None),
                        (EventType::Unparsed, Some("a")),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![
                        "== a ==",
                        "==  a  ==",
                        "==   a   ==",
                        " == a ==",
                        "== a == "
                    ],
                    vec![
                        (EventType::EnterHeading2, None),
                        (EventType::Unparsed, Some("a")),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec!["=== a ==="],
                    vec![
                        (EventType::EnterHeading3, None),
                        (EventType::Unparsed, Some("a")),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec!["==== a ===="],
                    vec![
                        (EventType::EnterHeading4, None),
                        (EventType::Unparsed, Some("a")),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec!["===== a ====="],
                    vec![
                        (EventType::EnterHeading5, None),
                        (EventType::Unparsed, Some("a")),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec!["====== a ======"],
                    vec![
                        (EventType::EnterHeading6, None),
                        (EventType::Unparsed, Some("a")),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec!["== a"],
                    vec![
                        (EventType::EnterHeading2, None),
                        (EventType::Unparsed, Some("a")),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec!["== a ="],
                    vec![
                        (EventType::EnterHeading2, None),
                        (EventType::Unparsed, Some("a =")),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec!["== a ==="],
                    vec![
                        (EventType::EnterHeading2, None),
                        (EventType::Unparsed, Some("a ===")),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![
                        indoc! {"
                        == a ==
                        b"},
                        indoc! {"
                        == a ==

                        b"},
                    ],
                    vec![
                        (EventType::EnterHeading2, None),
                        (EventType::Unparsed, Some("a")),
                        (EventType::ExitBlock, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("b")),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![
                        indoc! {"
                        == a ==
                        === b ==="},
                        indoc! {"
                        == a ==

                        === b ==="},
                    ],
                    vec![
                        (EventType::EnterHeading2, None),
                        (EventType::Unparsed, Some("a")),
                        (EventType::ExitBlock, None),
                        (EventType::EnterHeading3, None),
                        (EventType::Unparsed, Some("b")),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec!["==a =="],
                    vec![
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("==a ==")),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec!["== a=="],
                    vec![
                        (EventType::EnterHeading2, None),
                        (EventType::Unparsed, Some("a==")),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec!["== a ==b"],
                    vec![
                        (EventType::EnterHeading2, None),
                        (EventType::Unparsed, Some("a ==b")),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec!["== a == b =="],
                    vec![
                        (EventType::EnterHeading2, None),
                        (EventType::Unparsed, Some("a == b")),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec!["======= a ======="],
                    vec![
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("======= a =======")),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec!["== <`c`> =="],
                    vec![
                        (EventType::EnterHeading2, None),
                        (EventType::VerbatimEscaping, Some("c")),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec!["== a<`c`>b =="],
                    vec![
                        (EventType::EnterHeading2, None),
                        (EventType::Unparsed, Some("a")),
                        (EventType::VerbatimEscaping, Some("c")),
                        (EventType::Unparsed, Some("b")),
                        (EventType::ExitBlock, None),
                    ]
                ),
            ],
        },
        GroupedCases {
            group: "标题>无内容",
            cases: vec![
                case!(
                    vec!["=="],
                    vec![
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("==")),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec!["== =="],
                    vec![
                        (EventType::EnterHeading2, None),
                        (EventType::Unparsed, Some("==")),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec!["== ", "==  ==", "==   =="],
                    vec![
                        (EventType::EnterHeading2, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec!["== =", "==  =", "==   ="],
                    vec![
                        (EventType::EnterHeading2, None),
                        (EventType::Unparsed, Some("=")),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec!["== ===", "==  ===", "==   ==="],
                    vec![
                        (EventType::EnterHeading2, None),
                        (EventType::Unparsed, Some("===")),
                        (EventType::ExitBlock, None),
                    ]
                ),
            ],
        },
    ]
}
