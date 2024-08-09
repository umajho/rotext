use indoc::indoc;

use crate::{
    events::EventType,
    test_suites::block::support::{case, GroupedCases},
};

pub fn groups_block_quote() -> Vec<GroupedCases> {
    vec![
        GroupedCases {
            group: "块引用与其延续",
            cases: vec![
                case!(
                    vec![">", ">␠"],
                    vec![
                        (EventType::EnterBlockQuote, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec!["> foo", " > foo", ">  foo"],
                    vec![
                        (EventType::EnterBlockQuote, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("foo")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![
                        indoc! {"
                        > foo
                        > bar"},
                        indoc! {"
                        > foo
                        >  bar"},
                    ],
                    vec![
                        (EventType::EnterBlockQuote, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("foo")),
                        (EventType::NewLine, None),
                        (EventType::Unparsed, Some("bar")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![
                        indoc! {"
                        > foo
                        bar"},
                        indoc! {"
                        > foo
                         bar"},
                    ],
                    vec![
                        (EventType::EnterBlockQuote, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("foo")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("bar")),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![
                        indoc! {"
                        > foo

                        > bar"},
                        indoc! {"
                        > foo

                        >  bar"},
                    ],
                    vec![
                        (EventType::EnterBlockQuote, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("foo")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::EnterBlockQuote, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("bar")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec!["> > foo", " > > foo"],
                    vec![
                        (EventType::EnterBlockQuote, None),
                        (EventType::EnterBlockQuote, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("foo")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![
                        indoc! {"
                        > > foo
                        > bar"},
                        indoc! {"
                        > > foo
                        >  bar"},
                    ],
                    vec![
                        (EventType::EnterBlockQuote, None),
                        (EventType::EnterBlockQuote, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("foo")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("bar")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![indoc! {"
                        > foo
                        > > bar"},],
                    vec![
                        (EventType::EnterBlockQuote, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("foo")),
                        (EventType::NewLine, None),
                        (EventType::Unparsed, Some("> bar")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![
                        indoc! {"
                        >
                        > foo"},
                        indoc! {"
                        >␠
                        > foo"}
                    ],
                    vec![
                        (EventType::EnterBlockQuote, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("foo")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
            ],
        },
        GroupedCases {
            group: "块引用>块引用中的分割线与标题",
            cases: vec![
                case!(
                    vec!["> ---"],
                    vec![
                        (EventType::EnterBlockQuote, None),
                        (EventType::ThematicBreak, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![indoc! {"
                        > ---
                        ---"},],
                    vec![
                        (EventType::EnterBlockQuote, None),
                        (EventType::ThematicBreak, None),
                        (EventType::ExitBlock, None),
                        (EventType::ThematicBreak, None),
                    ]
                ),
                case!(
                    vec!["> == a =="],
                    vec![
                        (EventType::EnterBlockQuote, None),
                        (EventType::EnterHeading2, None),
                        (EventType::Unparsed, Some("a")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![indoc! {"
                        > == a ==
                        === b ==="},],
                    vec![
                        (EventType::EnterBlockQuote, None),
                        (EventType::EnterHeading2, None),
                        (EventType::Unparsed, Some("a")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::EnterHeading3, None),
                        (EventType::Unparsed, Some("b")),
                        (EventType::ExitBlock, None),
                    ]
                ),
            ],
        },
    ]
}
