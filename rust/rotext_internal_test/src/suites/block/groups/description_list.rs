use indoc::indoc;

use rotext_core::EventType;

use crate::suites::block::support::{case, GroupedCases};

pub fn groups_description_list() -> Vec<GroupedCases> {
    vec![GroupedCases {
        group: "描述列表",
        cases: vec![
            case!(
                vec!["; term"],
                vec![
                    (EventType::EnterDescriptionList, None),
                    (EventType::EnterDescriptionTerm, None),
                    (EventType::EnterParagraph, None),
                    (EventType::__Unparsed, Some("term")),
                    (EventType::ExitBlock, None),
                    (EventType::ExitBlock, None),
                    (EventType::ExitBlock, None),
                ]
            ),
            case!(
                vec![": details"],
                vec![
                    (EventType::EnterDescriptionList, None),
                    (EventType::EnterDescriptionDetails, None),
                    (EventType::EnterParagraph, None),
                    (EventType::__Unparsed, Some("details")),
                    (EventType::ExitBlock, None),
                    (EventType::ExitBlock, None),
                    (EventType::ExitBlock, None),
                ]
            ),
            case!(
                vec![
                    indoc! {"
                        ; term
                        : details"},
                    indoc! {"; term :: details"},
                ],
                vec![
                    (EventType::EnterDescriptionList, None),
                    (EventType::EnterDescriptionTerm, None),
                    (EventType::EnterParagraph, None),
                    (EventType::__Unparsed, Some("term")),
                    (EventType::ExitBlock, None),
                    (EventType::ExitBlock, None),
                    (EventType::EnterDescriptionDetails, None),
                    (EventType::EnterParagraph, None),
                    (EventType::__Unparsed, Some("details")),
                    (EventType::ExitBlock, None),
                    (EventType::ExitBlock, None),
                    (EventType::ExitBlock, None),
                ]
            ),
            case!(
                vec![indoc! {"
                    ; term
                    > term line 2
                    : details"},],
                vec![
                    (EventType::EnterDescriptionList, None),
                    (EventType::EnterDescriptionTerm, None),
                    (EventType::EnterParagraph, None),
                    (EventType::__Unparsed, Some("term")),
                    (EventType::NewLine, None),
                    (EventType::__Unparsed, Some("term line 2")),
                    (EventType::ExitBlock, None),
                    (EventType::ExitBlock, None),
                    (EventType::EnterDescriptionDetails, None),
                    (EventType::EnterParagraph, None),
                    (EventType::__Unparsed, Some("details")),
                    (EventType::ExitBlock, None),
                    (EventType::ExitBlock, None),
                    (EventType::ExitBlock, None),
                ]
            ),
            case!(
                vec![indoc! {"
                    ; term
                    > term line 2 :: details 1
                    : details 2"},],
                vec![
                    (EventType::EnterDescriptionList, None),
                    (EventType::EnterDescriptionTerm, None),
                    (EventType::EnterParagraph, None),
                    (EventType::__Unparsed, Some("term")),
                    (EventType::NewLine, None),
                    (EventType::__Unparsed, Some("term line 2")),
                    (EventType::ExitBlock, None),
                    (EventType::ExitBlock, None),
                    (EventType::EnterDescriptionDetails, None),
                    (EventType::EnterParagraph, None),
                    (EventType::__Unparsed, Some("details 1")),
                    (EventType::ExitBlock, None),
                    (EventType::ExitBlock, None),
                    (EventType::EnterDescriptionDetails, None),
                    (EventType::EnterParagraph, None),
                    (EventType::__Unparsed, Some("details 2")),
                    (EventType::ExitBlock, None),
                    (EventType::ExitBlock, None),
                    (EventType::ExitBlock, None),
                ]
            ),
            case!(
                vec![
                    indoc! {"
                        ; term
                        : details
                        > details line 2"},
                    indoc! {"
                        ; term :: details
                        > details line 2"},
                ],
                vec![
                    (EventType::EnterDescriptionList, None),
                    (EventType::EnterDescriptionTerm, None),
                    (EventType::EnterParagraph, None),
                    (EventType::__Unparsed, Some("term")),
                    (EventType::ExitBlock, None),
                    (EventType::ExitBlock, None),
                    (EventType::EnterDescriptionDetails, None),
                    (EventType::EnterParagraph, None),
                    (EventType::__Unparsed, Some("details")),
                    (EventType::NewLine, None),
                    (EventType::__Unparsed, Some("details line 2")),
                    (EventType::ExitBlock, None),
                    (EventType::ExitBlock, None),
                    (EventType::ExitBlock, None),
                ]
            ),
            case!(
                vec![
                    indoc! {"
                        ; term 1
                        : details 1
                        ; term 2
                        : details 2.1
                        : details 2.2"},
                    indoc! {"
                        ; term 1 :: details 1
                        ; term 2 :: details 2.1
                        : details 2.2"},
                ],
                vec![
                    (EventType::EnterDescriptionList, None),
                    (EventType::EnterDescriptionTerm, None),
                    (EventType::EnterParagraph, None),
                    (EventType::__Unparsed, Some("term 1")),
                    (EventType::ExitBlock, None),
                    (EventType::ExitBlock, None),
                    (EventType::EnterDescriptionDetails, None),
                    (EventType::EnterParagraph, None),
                    (EventType::__Unparsed, Some("details 1")),
                    (EventType::ExitBlock, None),
                    (EventType::ExitBlock, None),
                    (EventType::EnterDescriptionTerm, None),
                    (EventType::EnterParagraph, None),
                    (EventType::__Unparsed, Some("term 2")),
                    (EventType::ExitBlock, None),
                    (EventType::ExitBlock, None),
                    (EventType::EnterDescriptionDetails, None),
                    (EventType::EnterParagraph, None),
                    (EventType::__Unparsed, Some("details 2.1")),
                    (EventType::ExitBlock, None),
                    (EventType::ExitBlock, None),
                    (EventType::EnterDescriptionDetails, None),
                    (EventType::EnterParagraph, None),
                    (EventType::__Unparsed, Some("details 2.2")),
                    (EventType::ExitBlock, None),
                    (EventType::ExitBlock, None),
                    (EventType::ExitBlock, None),
                ]
            ),
            case!(
                vec![
                    indoc! {"; term :: "},
                    indoc! {"; term ::"},
                    indoc! {"; term ::\n"},
                ],
                vec![
                    (EventType::EnterDescriptionList, None),
                    (EventType::EnterDescriptionTerm, None),
                    (EventType::EnterParagraph, None),
                    (EventType::__Unparsed, Some("term")),
                    (EventType::ExitBlock, None),
                    (EventType::ExitBlock, None),
                    (EventType::EnterDescriptionDetails, None),
                    (EventType::ExitBlock, None),
                    (EventType::ExitBlock, None),
                ]
            ),
            case!(
                vec![indoc! {"
                        ; term 1 ::
                        > ; term 1.1 :: details 1.1a
                        > : details 1.1b
                        > ; term 1.2"},],
                vec![
                    (EventType::EnterDescriptionList, None),
                    (EventType::EnterDescriptionTerm, None),
                    (EventType::EnterParagraph, None),
                    (EventType::__Unparsed, Some("term 1")),
                    (EventType::ExitBlock, None),
                    (EventType::ExitBlock, None),
                    (EventType::EnterDescriptionDetails, None),
                    (EventType::EnterDescriptionList, None),
                    (EventType::EnterDescriptionTerm, None),
                    (EventType::EnterParagraph, None),
                    (EventType::__Unparsed, Some("term 1.1")),
                    (EventType::ExitBlock, None),
                    (EventType::ExitBlock, None),
                    (EventType::EnterDescriptionDetails, None),
                    (EventType::EnterParagraph, None),
                    (EventType::__Unparsed, Some("details 1.1a")),
                    (EventType::ExitBlock, None),
                    (EventType::ExitBlock, None),
                    (EventType::EnterDescriptionDetails, None),
                    (EventType::EnterParagraph, None),
                    (EventType::__Unparsed, Some("details 1.1b")),
                    (EventType::ExitBlock, None),
                    (EventType::ExitBlock, None),
                    (EventType::EnterDescriptionTerm, None),
                    (EventType::EnterParagraph, None),
                    (EventType::__Unparsed, Some("term 1.2")),
                    (EventType::ExitBlock, None),
                    (EventType::ExitBlock, None),
                    (EventType::ExitBlock, None),
                    (EventType::ExitBlock, None),
                    (EventType::ExitBlock, None),
                ]
            ),
            case!(
                vec![indoc! {"
                    ; ; term 1
                    > : details 1
                    : ; term 2"},],
                vec![
                    (EventType::EnterDescriptionList, None),
                    (EventType::EnterDescriptionTerm, None),
                    (EventType::EnterDescriptionList, None),
                    (EventType::EnterDescriptionTerm, None),
                    (EventType::EnterParagraph, None),
                    (EventType::__Unparsed, Some("term 1")),
                    (EventType::ExitBlock, None),
                    (EventType::ExitBlock, None),
                    (EventType::EnterDescriptionDetails, None),
                    (EventType::EnterParagraph, None),
                    (EventType::__Unparsed, Some("details 1")),
                    (EventType::ExitBlock, None),
                    (EventType::ExitBlock, None),
                    (EventType::ExitBlock, None),
                    (EventType::ExitBlock, None),
                    (EventType::EnterDescriptionDetails, None),
                    (EventType::EnterDescriptionList, None),
                    (EventType::EnterDescriptionTerm, None),
                    (EventType::EnterParagraph, None),
                    (EventType::__Unparsed, Some("term 2")),
                    (EventType::ExitBlock, None),
                    (EventType::ExitBlock, None),
                    (EventType::ExitBlock, None),
                    (EventType::ExitBlock, None),
                    (EventType::ExitBlock, None),
                ]
            ),
            case!(
                vec![indoc! {"
                    ; # a
                    > > b
                    : > foo"},],
                vec![
                    (EventType::EnterDescriptionList, None),
                    (EventType::EnterDescriptionTerm, None),
                    (EventType::EnterOrderedList, None),
                    (EventType::EnterListItem, None),
                    (EventType::EnterParagraph, None),
                    (EventType::__Unparsed, Some("a")),
                    (EventType::NewLine, None),
                    (EventType::__Unparsed, Some("b")),
                    (EventType::ExitBlock, None),
                    (EventType::ExitBlock, None),
                    (EventType::ExitBlock, None),
                    (EventType::ExitBlock, None),
                    (EventType::EnterDescriptionDetails, None),
                    (EventType::EnterBlockQuote, None),
                    (EventType::EnterParagraph, None),
                    (EventType::__Unparsed, Some("foo")),
                    (EventType::ExitBlock, None),
                    (EventType::ExitBlock, None),
                    (EventType::ExitBlock, None),
                    (EventType::ExitBlock, None),
                ]
            ),
            case!(
                vec![
                    indoc! {"
                        > ; term 1 :: details 1
                        > ; term 2 :: details 2"},
                    indoc! {"
                        >
                        > ; term 1 :: details 1
                        > ; term 2 :: details 2"},
                    indoc! {"
                        > ; term 1
                        > : details 1
                        > ; term 2
                        > : details 2"},
                ],
                vec![
                    (EventType::EnterBlockQuote, None),
                    (EventType::EnterDescriptionList, None),
                    (EventType::EnterDescriptionTerm, None),
                    (EventType::EnterParagraph, None),
                    (EventType::__Unparsed, Some("term 1")),
                    (EventType::ExitBlock, None),
                    (EventType::ExitBlock, None),
                    (EventType::EnterDescriptionDetails, None),
                    (EventType::EnterParagraph, None),
                    (EventType::__Unparsed, Some("details 1")),
                    (EventType::ExitBlock, None),
                    (EventType::ExitBlock, None),
                    (EventType::EnterDescriptionTerm, None),
                    (EventType::EnterParagraph, None),
                    (EventType::__Unparsed, Some("term 2")),
                    (EventType::ExitBlock, None),
                    (EventType::ExitBlock, None),
                    (EventType::EnterDescriptionDetails, None),
                    (EventType::EnterParagraph, None),
                    (EventType::__Unparsed, Some("details 2")),
                    (EventType::ExitBlock, None),
                    (EventType::ExitBlock, None),
                    (EventType::ExitBlock, None),
                    (EventType::ExitBlock, None),
                ]
            ),
        ],
    }]
}
