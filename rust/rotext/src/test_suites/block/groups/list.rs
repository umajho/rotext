use indoc::indoc;

use crate::{
    events::EventType,
    test_suites::block::support::{case, GroupedCases},
};

pub fn groups_list() -> Vec<GroupedCases> {
    vec![
        GroupedCases {
            group: "列表",
            cases: vec![
                case!(
                    vec!["# 1"],
                    vec![
                        (EventType::EnterOrderedList, None),
                        (EventType::EnterListItem, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("1")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec!["* 1"],
                    vec![
                        (EventType::EnterUnorderedList, None),
                        (EventType::EnterListItem, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("1")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![indoc! {"
                        # 1
                        # 2"},],
                    vec![
                        (EventType::EnterOrderedList, None),
                        (EventType::EnterListItem, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("1")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::EnterListItem, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("2")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![indoc! {"
                        * a
                        * b
                        * c"},],
                    vec![
                        (EventType::EnterUnorderedList, None),
                        (EventType::EnterListItem, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("a")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::EnterListItem, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("b")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::EnterListItem, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("c")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![indoc! {"
                        # 1

                        # 2"},],
                    vec![
                        (EventType::EnterOrderedList, None),
                        (EventType::EnterListItem, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("1")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::EnterOrderedList, None),
                        (EventType::EnterListItem, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("2")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec!["# # 1.1"],
                    vec![
                        (EventType::EnterOrderedList, None),
                        (EventType::EnterListItem, None),
                        (EventType::EnterOrderedList, None),
                        (EventType::EnterListItem, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("1.1")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![indoc! {"
                        # # 1.1
                        # 2"},],
                    vec![
                        (EventType::EnterOrderedList, None),
                        (EventType::EnterListItem, None),
                        (EventType::EnterOrderedList, None),
                        (EventType::EnterListItem, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("1.1")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::EnterListItem, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("2")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![indoc! {"
                        # # 1.1
                        #
                        # 2"},],
                    vec![
                        (EventType::EnterOrderedList, None),
                        (EventType::EnterListItem, None),
                        (EventType::EnterOrderedList, None),
                        (EventType::EnterListItem, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("1.1")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::EnterListItem, None),
                        (EventType::ExitBlock, None),
                        (EventType::EnterListItem, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("2")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![indoc! {"
                        # 1
                        # # 2.1"},],
                    vec![
                        (EventType::EnterOrderedList, None),
                        (EventType::EnterListItem, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("1")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::EnterListItem, None),
                        (EventType::EnterOrderedList, None),
                        (EventType::EnterListItem, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("2.1")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![indoc! {"
                        # 1
                        #
                        # # 2.1"},],
                    vec![
                        (EventType::EnterOrderedList, None),
                        (EventType::EnterListItem, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("1")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::EnterListItem, None),
                        (EventType::ExitBlock, None),
                        (EventType::EnterListItem, None),
                        (EventType::EnterOrderedList, None),
                        (EventType::EnterListItem, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("2.1")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![indoc! {"
                        # 1
                        # # 2.1
                        # 3"},],
                    vec![
                        (EventType::EnterOrderedList, None),
                        (EventType::EnterListItem, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("1")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::EnterListItem, None),
                        (EventType::EnterOrderedList, None),
                        (EventType::EnterListItem, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("2.1")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::EnterListItem, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("3")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![indoc! {"
                        # # 1.1
                        # 2
                        # # 3.1"},],
                    vec![
                        (EventType::EnterOrderedList, None),
                        (EventType::EnterListItem, None),
                        (EventType::EnterOrderedList, None),
                        (EventType::EnterListItem, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("1.1")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::EnterListItem, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("2")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::EnterListItem, None),
                        (EventType::EnterOrderedList, None),
                        (EventType::EnterListItem, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("3.1")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
            ],
        },
        GroupedCases {
            group: "列表>不同种类的列表",
            cases: vec![case!(
                vec![
                    indoc! {"
                        # 1
                        * a"},
                    // indock},
                ],
                vec![
                    (EventType::EnterOrderedList, None),
                    (EventType::EnterListItem, None),
                    (EventType::EnterParagraph, None),
                    (EventType::Unparsed, Some("1")),
                    (EventType::ExitBlock, None),
                    (EventType::ExitBlock, None),
                    (EventType::ExitBlock, None),
                    (EventType::EnterUnorderedList, None),
                    (EventType::EnterListItem, None),
                    (EventType::EnterParagraph, None),
                    (EventType::Unparsed, Some("a")),
                    (EventType::ExitBlock, None),
                    (EventType::ExitBlock, None),
                    (EventType::ExitBlock, None),
                ]
            )],
        },
        GroupedCases {
            group: "列表>列表的延续",
            cases: vec![
                case!(
                    vec![indoc! {"
                        # a
                        > b"},],
                    vec![
                        (EventType::EnterOrderedList, None),
                        (EventType::EnterListItem, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("a")),
                        (EventType::NewLine, None),
                        (EventType::Unparsed, Some("b")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![indoc! {"
                        # a
                        >
                        > b"},],
                    vec![
                        (EventType::EnterOrderedList, None),
                        (EventType::EnterListItem, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("a")),
                        (EventType::ExitBlock, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("b")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![indoc! {"
                        # # 1.1
                        > # 1.2"},],
                    vec![
                        (EventType::EnterOrderedList, None),
                        (EventType::EnterListItem, None),
                        (EventType::EnterOrderedList, None),
                        (EventType::EnterListItem, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("1.1")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::EnterListItem, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("1.2")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![indoc! {"
                        # # a
                        > > b"},],
                    vec![
                        (EventType::EnterOrderedList, None),
                        (EventType::EnterListItem, None),
                        (EventType::EnterOrderedList, None),
                        (EventType::EnterListItem, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("a")),
                        (EventType::NewLine, None),
                        (EventType::Unparsed, Some("b")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
            ],
        },
        GroupedCases {
            group: "列表>列表或块引用的下一行的第一个字符可能与列表相关，但因为缺少空格而实际无关",
            cases: vec![
                case!(
                    vec![indoc! {"
                        # 1
                        #foo"},],
                    vec![
                        (EventType::EnterOrderedList, None),
                        (EventType::EnterListItem, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("1")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("#foo")),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![indoc! {"
                        > foo
                        #bar"},],
                    vec![
                        (EventType::EnterBlockQuote, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("foo")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("#bar")),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![indoc! {"
                        * a
                        #bar"},],
                    vec![
                        (EventType::EnterUnorderedList, None),
                        (EventType::EnterListItem, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("a")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("#bar")),
                        (EventType::ExitBlock, None),
                    ]
                ),
            ],
        },
        GroupedCases {
            group: "列表>列表中的先前测试过的块级元素",
            cases: vec![
                case!(
                    vec![indoc! {"
                        == a ==
                        # 1"},],
                    vec![
                        (EventType::EnterHeading2, None),
                        (EventType::Unparsed, Some("a")),
                        (EventType::ExitBlock, None),
                        (EventType::EnterOrderedList, None),
                        (EventType::EnterListItem, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("1")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![indoc! {"
                    # == a =="},],
                    vec![
                        (EventType::EnterOrderedList, None),
                        (EventType::EnterListItem, None),
                        (EventType::EnterHeading2, None),
                        (EventType::Unparsed, Some("a")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![indoc! {"
                        # == a ==
                        > b"},],
                    vec![
                        (EventType::EnterOrderedList, None),
                        (EventType::EnterListItem, None),
                        (EventType::EnterHeading2, None),
                        (EventType::Unparsed, Some("a")),
                        (EventType::ExitBlock, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("b")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![indoc! {"
                        # == a ==
                        # b"},],
                    vec![
                        (EventType::EnterOrderedList, None),
                        (EventType::EnterListItem, None),
                        (EventType::EnterHeading2, None),
                        (EventType::Unparsed, Some("a")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::EnterListItem, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("b")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![indoc! {"
                        > foo
                        # 1"},],
                    vec![
                        (EventType::EnterBlockQuote, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("foo")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::EnterOrderedList, None),
                        (EventType::EnterListItem, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("1")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![indoc! {"
                        # > foo
                        # # 1.1
                        > # 1.2"},],
                    vec![
                        (EventType::EnterOrderedList, None),
                        (EventType::EnterListItem, None),
                        (EventType::EnterBlockQuote, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("foo")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::EnterListItem, None),
                        (EventType::EnterOrderedList, None),
                        (EventType::EnterListItem, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("1.1")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::EnterListItem, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("1.2")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![indoc! {"
                        # # 1.1
                        >
                        > > foo"},],
                    vec![
                        (EventType::EnterOrderedList, None),
                        (EventType::EnterListItem, None),
                        (EventType::EnterOrderedList, None),
                        (EventType::EnterListItem, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("1.1")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::EnterBlockQuote, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("foo")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
            ],
        },
    ]
}
