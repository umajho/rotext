use indoc::indoc;

use rotext_core::EventType;

use crate::suites::block::support::{GroupedCases, case};

pub fn groups_block_quote() -> Vec<GroupedCases> {
    vec![
        GroupedCases {
            group: "块引用与其延续",
            cases: vec![
                case!(vec![">", ">␠"], vec![
                    (EventType::EnterBlockQuote, None),
                    (EventType::ExitBlock, None),
                ]),
                case!(vec![">␠foo", "␠>␠foo", ">␠␠foo"], vec![
                    (EventType::EnterBlockQuote, None),
                    (EventType::EnterParagraph, None),
                    (EventType::__Unparsed, Some("foo")),
                    (EventType::ExitBlock, None),
                    (EventType::ExitBlock, None),
                ]),
                case!(
                    vec![
                        indoc! {"
                        >␠foo
                        >␠bar"},
                        indoc! {"
                        >␠foo
                        >␠␠bar"},
                    ],
                    vec![
                        (EventType::EnterBlockQuote, None),
                        (EventType::EnterParagraph, None),
                        (EventType::__Unparsed, Some("foo")),
                        (EventType::NewLine, None),
                        (EventType::__Unparsed, Some("bar")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![
                        indoc! {"
                        >␠foo
                        bar"},
                        indoc! {"
                        >␠foo
                        ␠bar"},
                    ],
                    vec![
                        (EventType::EnterBlockQuote, None),
                        (EventType::EnterParagraph, None),
                        (EventType::__Unparsed, Some("foo")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::EnterParagraph, None),
                        (EventType::__Unparsed, Some("bar")),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![
                        indoc! {"
                        >␠foo

                        >␠bar"},
                        indoc! {"
                        >␠foo

                        >␠␠bar"},
                    ],
                    vec![
                        (EventType::EnterBlockQuote, None),
                        (EventType::EnterParagraph, None),
                        (EventType::__Unparsed, Some("foo")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::EnterBlockQuote, None),
                        (EventType::EnterParagraph, None),
                        (EventType::__Unparsed, Some("bar")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(vec![">␠>␠foo", "␠>␠>␠foo"], vec![
                    (EventType::EnterBlockQuote, None),
                    (EventType::EnterBlockQuote, None),
                    (EventType::EnterParagraph, None),
                    (EventType::__Unparsed, Some("foo")),
                    (EventType::ExitBlock, None),
                    (EventType::ExitBlock, None),
                    (EventType::ExitBlock, None),
                ]),
                case!(
                    vec![
                        indoc! {"
                        >␠>␠foo
                        >␠bar"},
                        indoc! {"
                        >␠>␠foo
                        >␠␠bar"},
                    ],
                    vec![
                        (EventType::EnterBlockQuote, None),
                        (EventType::EnterBlockQuote, None),
                        (EventType::EnterParagraph, None),
                        (EventType::__Unparsed, Some("foo")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::EnterParagraph, None),
                        (EventType::__Unparsed, Some("bar")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![indoc! {"
                        >␠foo
                        >␠>␣bar"},],
                    vec![
                        (EventType::EnterBlockQuote, None),
                        (EventType::EnterParagraph, None),
                        (EventType::__Unparsed, Some("foo")),
                        (EventType::NewLine, None),
                        (EventType::__Unparsed, Some("> bar")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![
                        indoc! {"
                        >
                        >␠foo"},
                        indoc! {"
                        >␠
                        >␠foo"}
                    ],
                    vec![
                        (EventType::EnterBlockQuote, None),
                        (EventType::EnterParagraph, None),
                        (EventType::__Unparsed, Some("foo")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
            ],
        },
        GroupedCases {
            group: "块引用>块引用中的分割线与标题",
            cases: vec![
                case!(vec![">␠---"], vec![
                    (EventType::EnterBlockQuote, None),
                    (EventType::ThematicBreak, None),
                    (EventType::ExitBlock, None),
                ]),
                case!(
                    vec![indoc! {"
                        >␠---
                        ---"},],
                    vec![
                        (EventType::EnterBlockQuote, None),
                        (EventType::ThematicBreak, None),
                        (EventType::ExitBlock, None),
                        (EventType::ThematicBreak, None),
                    ]
                ),
                case!(vec![">␠==␠a␠=="], vec![
                    (EventType::EnterBlockQuote, None),
                    (EventType::EnterHeading2, None),
                    (EventType::__Unparsed, Some("a")),
                    (EventType::ExitBlock, None),
                    (EventType::ExitBlock, None),
                ]),
                case!(
                    vec![indoc! {"
                        >␠==␠a␠==
                        ===␠b␠==="},],
                    vec![
                        (EventType::EnterBlockQuote, None),
                        (EventType::EnterHeading2, None),
                        (EventType::__Unparsed, Some("a")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::EnterHeading3, None),
                        (EventType::__Unparsed, Some("b")),
                        (EventType::ExitBlock, None),
                    ]
                ),
            ],
        },
        GroupedCases {
            group: "块引用>块引用的下一行的第一个字符有可能延续块引用，但因为缺少空格而未能如此",
            cases: vec![case!(
                vec![indoc! {"
                    >␠foo
                    >bar"},],
                vec![
                    (EventType::EnterBlockQuote, None),
                    (EventType::EnterParagraph, None),
                    (EventType::__Unparsed, Some("foo")),
                    (EventType::ExitBlock, None),
                    (EventType::ExitBlock, None),
                    (EventType::EnterParagraph, None),
                    (EventType::__Unparsed, Some(">bar")),
                    (EventType::ExitBlock, None),
                ]
            )],
        },
    ]
}
