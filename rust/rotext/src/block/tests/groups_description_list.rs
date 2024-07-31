use indoc::indoc;

use crate::{
    block::tests::{support::case, GroupedCases},
    events::EventType,
};

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
                    (EventType::Unparsed, Some("term")),
                    (EventType::ExitBlock, None),
                    (EventType::ExitBlock, None),
                    (EventType::ExitBlock, None)
                ]
            ),
            case!(
                vec![": details"],
                vec![
                    (EventType::EnterDescriptionList, None),
                    (EventType::EnterDescriptionDetails, None),
                    (EventType::EnterParagraph, None),
                    (EventType::Unparsed, Some("details")),
                    (EventType::ExitBlock, None),
                    (EventType::ExitBlock, None),
                    (EventType::ExitBlock, None)
                ]
            ),
            case!(
                vec![indoc! {"
                    ; term
                    : details"},],
                vec![
                    (EventType::EnterDescriptionList, None),
                    (EventType::EnterDescriptionTerm, None),
                    (EventType::EnterParagraph, None),
                    (EventType::Unparsed, Some("term")),
                    (EventType::ExitBlock, None),
                    (EventType::ExitBlock, None),
                    (EventType::EnterDescriptionDetails, None),
                    (EventType::EnterParagraph, None),
                    (EventType::Unparsed, Some("details")),
                    (EventType::ExitBlock, None),
                    (EventType::ExitBlock, None),
                    (EventType::ExitBlock, None)
                ]
            ),
            case!(
                vec![indoc! {"
                    ; term 1
                    : details 1
                    ; term 2
                    : details 2.1
                    : details 2.2"},],
                vec![
                    (EventType::EnterDescriptionList, None),
                    (EventType::EnterDescriptionTerm, None),
                    (EventType::EnterParagraph, None),
                    (EventType::Unparsed, Some("term 1")),
                    (EventType::ExitBlock, None),
                    (EventType::ExitBlock, None),
                    (EventType::EnterDescriptionDetails, None),
                    (EventType::EnterParagraph, None),
                    (EventType::Unparsed, Some("details 1")),
                    (EventType::ExitBlock, None),
                    (EventType::ExitBlock, None),
                    (EventType::EnterDescriptionTerm, None),
                    (EventType::EnterParagraph, None),
                    (EventType::Unparsed, Some("term 2")),
                    (EventType::ExitBlock, None),
                    (EventType::ExitBlock, None),
                    (EventType::EnterDescriptionDetails, None),
                    (EventType::EnterParagraph, None),
                    (EventType::Unparsed, Some("details 2.1")),
                    (EventType::ExitBlock, None),
                    (EventType::ExitBlock, None),
                    (EventType::EnterDescriptionDetails, None),
                    (EventType::EnterParagraph, None),
                    (EventType::Unparsed, Some("details 2.2")),
                    (EventType::ExitBlock, None),
                    (EventType::ExitBlock, None),
                    (EventType::ExitBlock, None)
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
                    (EventType::Unparsed, Some("term 1")),
                    (EventType::ExitBlock, None),
                    (EventType::ExitBlock, None),
                    (EventType::EnterDescriptionDetails, None),
                    (EventType::EnterParagraph, None),
                    (EventType::Unparsed, Some("details 1")),
                    (EventType::ExitBlock, None),
                    (EventType::ExitBlock, None),
                    (EventType::ExitBlock, None),
                    (EventType::ExitBlock, None),
                    (EventType::EnterDescriptionDetails, None),
                    (EventType::EnterDescriptionList, None),
                    (EventType::EnterDescriptionTerm, None),
                    (EventType::EnterParagraph, None),
                    (EventType::Unparsed, Some("term 2")),
                    (EventType::ExitBlock, None),
                    (EventType::ExitBlock, None),
                    (EventType::ExitBlock, None),
                    (EventType::ExitBlock, None),
                    (EventType::ExitBlock, None)
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
                    (EventType::Unparsed, Some("a")),
                    (EventType::NewLine, None),
                    (EventType::Unparsed, Some("b")),
                    (EventType::ExitBlock, None),
                    (EventType::ExitBlock, None),
                    (EventType::ExitBlock, None),
                    (EventType::ExitBlock, None),
                    (EventType::EnterDescriptionDetails, None),
                    (EventType::EnterBlockQuote, None),
                    (EventType::EnterParagraph, None),
                    (EventType::Unparsed, Some("foo")),
                    (EventType::ExitBlock, None),
                    (EventType::ExitBlock, None),
                    (EventType::ExitBlock, None),
                    (EventType::ExitBlock, None)
                ]
            ),
        ],
    }]
}
