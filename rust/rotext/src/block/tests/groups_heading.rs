use indoc::indoc;

use crate::{
    block::tests::{utils::case, GroupedCases},
    events::EventType,
};

pub fn groups_heading() -> Vec<GroupedCases> {
    vec![GroupedCases {
        group: "标题",
        cases: vec![
            case!(
                vec!["= a ="],
                vec![
                    (EventType::EnterHeading1, None),
                    (EventType::Unparsed, Some("a")),
                    (EventType::Exit, None)
                ]
            ),
            case!(
                vec!["== a =="],
                vec![
                    (EventType::EnterHeading2, None),
                    (EventType::Unparsed, Some("a")),
                    (EventType::Exit, None)
                ]
            ),
            case!(
                vec!["=== a ==="],
                vec![
                    (EventType::EnterHeading3, None),
                    (EventType::Unparsed, Some("a")),
                    (EventType::Exit, None)
                ]
            ),
            case!(
                vec!["==== a ===="],
                vec![
                    (EventType::EnterHeading4, None),
                    (EventType::Unparsed, Some("a")),
                    (EventType::Exit, None)
                ]
            ),
            case!(
                vec!["===== a ====="],
                vec![
                    (EventType::EnterHeading5, None),
                    (EventType::Unparsed, Some("a")),
                    (EventType::Exit, None)
                ]
            ),
            case!(
                vec!["====== a ======"],
                vec![
                    (EventType::EnterHeading6, None),
                    (EventType::Unparsed, Some("a")),
                    (EventType::Exit, None)
                ]
            ),
            case!(
                vec!["== a"],
                vec![
                    (EventType::EnterHeading2, None),
                    (EventType::Unparsed, Some("a")),
                    (EventType::Exit, None)
                ]
            ),
            case!(
                vec!["== a ="],
                vec![
                    (EventType::EnterHeading2, None),
                    (EventType::Unparsed, Some("a =")),
                    (EventType::Exit, None)
                ]
            ),
            case!(
                vec!["== a ==="],
                vec![
                    (EventType::EnterHeading2, None),
                    (EventType::Unparsed, Some("a ===")),
                    (EventType::Exit, None)
                ]
            ),
            case!(
                vec!["==  a  =="],
                vec![
                    (EventType::EnterHeading2, None),
                    (EventType::Unparsed, Some(" a ")),
                    (EventType::Exit, None)
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
                    (EventType::Exit, None),
                    (EventType::EnterParagraph, None),
                    (EventType::Unparsed, Some("b")),
                    (EventType::Exit, None)
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
                    (EventType::Exit, None),
                    (EventType::EnterHeading3, None),
                    (EventType::Unparsed, Some("b")),
                    (EventType::Exit, None)
                ]
            ),
            case!(
                vec!["==a =="],
                vec![
                    (EventType::EnterParagraph, None),
                    (EventType::Unparsed, Some("==a ==")),
                    (EventType::Exit, None)
                ]
            ),
            case!(
                vec!["== a=="],
                vec![
                    (EventType::EnterHeading2, None),
                    (EventType::Unparsed, Some("a==")),
                    (EventType::Exit, None)
                ]
            ),
            case!(
                vec!["== a == "],
                vec![
                    (EventType::EnterHeading2, None),
                    (EventType::Unparsed, Some("a == ")),
                    (EventType::Exit, None)
                ]
            ),
            case!(
                vec!["== a ==b"],
                vec![
                    (EventType::EnterHeading2, None),
                    (EventType::Unparsed, Some("a ==b")),
                    (EventType::Exit, None)
                ]
            ),
            case!(
                vec!["== a == b =="],
                vec![
                    (EventType::EnterHeading2, None),
                    (EventType::Unparsed, Some("a == b")),
                    (EventType::Exit, None)
                ]
            ),
            case!(
                vec!["======= a ======="],
                vec![
                    (EventType::EnterParagraph, None),
                    (EventType::Unparsed, Some("======= a =======")),
                    (EventType::Exit, None)
                ]
            ),
            case!(
                vec!["== <`c`> =="],
                vec![
                    (EventType::EnterHeading2, None),
                    (EventType::VerbatimEscaping, Some("c")),
                    (EventType::Exit, None)
                ]
            ),
            case!(
                vec!["== a<`c`>b =="],
                vec![
                    (EventType::EnterHeading2, None),
                    (EventType::Unparsed, Some("a")),
                    (EventType::VerbatimEscaping, Some("c")),
                    (EventType::Unparsed, Some("b")),
                    (EventType::Exit, None)
                ]
            ),
        ],
    }]
}
