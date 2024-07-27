use indoc::indoc;

use crate::{
    block::tests::{utils::case, GroupedCases},
    events::EventType,
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
                        (EventType::Exit, None),
                        (EventType::Exit, None),
                        (EventType::Exit, None)
                    ]
                ),
                case!(
                    vec!["* 1"],
                    vec![
                        (EventType::EnterUnorderedList, None),
                        (EventType::EnterListItem, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("1")),
                        (EventType::Exit, None),
                        (EventType::Exit, None),
                        (EventType::Exit, None)
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
                        (EventType::Exit, None),
                        (EventType::Exit, None),
                        (EventType::EnterListItem, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("2")),
                        (EventType::Exit, None),
                        (EventType::Exit, None),
                        (EventType::Exit, None)
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
                        (EventType::Exit, None),
                        (EventType::Exit, None),
                        (EventType::EnterListItem, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("b")),
                        (EventType::Exit, None),
                        (EventType::Exit, None),
                        (EventType::EnterListItem, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("c")),
                        (EventType::Exit, None),
                        (EventType::Exit, None),
                        (EventType::Exit, None)
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
                        (EventType::Exit, None),
                        (EventType::Exit, None),
                        (EventType::Exit, None),
                        (EventType::EnterOrderedList, None),
                        (EventType::EnterListItem, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("2")),
                        (EventType::Exit, None),
                        (EventType::Exit, None),
                        (EventType::Exit, None)
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
                        (EventType::Exit, None),
                        (EventType::Exit, None),
                        (EventType::Exit, None),
                        (EventType::Exit, None),
                        (EventType::Exit, None)
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
                        (EventType::Exit, None),
                        (EventType::Exit, None),
                        (EventType::Exit, None),
                        (EventType::Exit, None),
                        (EventType::EnterListItem, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("2")),
                        (EventType::Exit, None),
                        (EventType::Exit, None),
                        (EventType::Exit, None)
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
                        (EventType::Exit, None),
                        (EventType::Exit, None),
                        (EventType::Exit, None),
                        (EventType::Exit, None),
                        (EventType::EnterListItem, None),
                        (EventType::Exit, None),
                        (EventType::EnterListItem, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("2")),
                        (EventType::Exit, None),
                        (EventType::Exit, None),
                        (EventType::Exit, None)
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
                        (EventType::Exit, None),
                        (EventType::Exit, None),
                        (EventType::EnterListItem, None),
                        (EventType::EnterOrderedList, None),
                        (EventType::EnterListItem, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("2.1")),
                        (EventType::Exit, None),
                        (EventType::Exit, None),
                        (EventType::Exit, None),
                        (EventType::Exit, None),
                        (EventType::Exit, None)
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
                        (EventType::Exit, None),
                        (EventType::Exit, None),
                        (EventType::EnterListItem, None),
                        (EventType::Exit, None),
                        (EventType::EnterListItem, None),
                        (EventType::EnterOrderedList, None),
                        (EventType::EnterListItem, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("2.1")),
                        (EventType::Exit, None),
                        (EventType::Exit, None),
                        (EventType::Exit, None),
                        (EventType::Exit, None),
                        (EventType::Exit, None)
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
                        (EventType::Exit, None),
                        (EventType::Exit, None),
                        (EventType::EnterListItem, None),
                        (EventType::EnterOrderedList, None),
                        (EventType::EnterListItem, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("2.1")),
                        (EventType::Exit, None),
                        (EventType::Exit, None),
                        (EventType::Exit, None),
                        (EventType::Exit, None),
                        (EventType::EnterListItem, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("3")),
                        (EventType::Exit, None),
                        (EventType::Exit, None),
                        (EventType::Exit, None)
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
                        (EventType::Exit, None),
                        (EventType::Exit, None),
                        (EventType::Exit, None),
                        (EventType::Exit, None),
                        (EventType::EnterListItem, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("2")),
                        (EventType::Exit, None),
                        (EventType::Exit, None),
                        (EventType::EnterListItem, None),
                        (EventType::EnterOrderedList, None),
                        (EventType::EnterListItem, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("3.1")),
                        (EventType::Exit, None),
                        (EventType::Exit, None),
                        (EventType::Exit, None),
                        (EventType::Exit, None),
                        (EventType::Exit, None)
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
                    indoc! {"
                                # 1

                                * a"},
                ],
                vec![
                    (EventType::EnterOrderedList, None),
                    (EventType::EnterListItem, None),
                    (EventType::EnterParagraph, None),
                    (EventType::Unparsed, Some("1")),
                    (EventType::Exit, None),
                    (EventType::Exit, None),
                    (EventType::Exit, None),
                    (EventType::EnterUnorderedList, None),
                    (EventType::EnterListItem, None),
                    (EventType::EnterParagraph, None),
                    (EventType::Unparsed, Some("a")),
                    (EventType::Exit, None),
                    (EventType::Exit, None),
                    (EventType::Exit, None)
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
                        (EventType::Exit, None),
                        (EventType::Exit, None),
                        (EventType::Exit, None)
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
                        (EventType::Exit, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("b")),
                        (EventType::Exit, None),
                        (EventType::Exit, None),
                        (EventType::Exit, None)
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
                        (EventType::Exit, None),
                        (EventType::Exit, None),
                        (EventType::EnterListItem, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("1.2")),
                        (EventType::Exit, None),
                        (EventType::Exit, None),
                        (EventType::Exit, None),
                        (EventType::Exit, None),
                        (EventType::Exit, None)
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
                        (EventType::Exit, None),
                        (EventType::Exit, None),
                        (EventType::Exit, None),
                        (EventType::Exit, None),
                        (EventType::Exit, None)
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
                        (EventType::Exit, None),
                        (EventType::EnterOrderedList, None),
                        (EventType::EnterListItem, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("1")),
                        (EventType::Exit, None),
                        (EventType::Exit, None),
                        (EventType::Exit, None)
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
                        (EventType::Exit, None),
                        (EventType::Exit, None),
                        (EventType::Exit, None)
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
                        (EventType::Exit, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("b")),
                        (EventType::Exit, None),
                        (EventType::Exit, None),
                        (EventType::Exit, None)
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
                        (EventType::Exit, None),
                        (EventType::Exit, None),
                        (EventType::EnterListItem, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("b")),
                        (EventType::Exit, None),
                        (EventType::Exit, None),
                        (EventType::Exit, None)
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
                        (EventType::Exit, None),
                        (EventType::Exit, None),
                        (EventType::EnterOrderedList, None),
                        (EventType::EnterListItem, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("1")),
                        (EventType::Exit, None),
                        (EventType::Exit, None),
                        (EventType::Exit, None)
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
                        (EventType::Exit, None),
                        (EventType::Exit, None),
                        (EventType::Exit, None),
                        (EventType::EnterListItem, None),
                        (EventType::EnterOrderedList, None),
                        (EventType::EnterListItem, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("1.1")),
                        (EventType::Exit, None),
                        (EventType::Exit, None),
                        (EventType::EnterListItem, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("1.2")),
                        (EventType::Exit, None),
                        (EventType::Exit, None),
                        (EventType::Exit, None),
                        (EventType::Exit, None),
                        (EventType::Exit, None)
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
                        (EventType::Exit, None),
                        (EventType::Exit, None),
                        (EventType::Exit, None),
                        (EventType::EnterBlockQuote, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("foo")),
                        (EventType::Exit, None),
                        (EventType::Exit, None),
                        (EventType::Exit, None),
                        (EventType::Exit, None)
                    ]
                ),
            ],
        },
    ]
}
