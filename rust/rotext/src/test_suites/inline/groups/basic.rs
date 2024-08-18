use indoc::indoc;

use crate::{
    events::EventType,
    test_suites::inline::support::{case, GroupedCases},
};

pub fn groups_basic() -> Vec<GroupedCases> {
    vec![GroupedCases {
        group: "基础",
        cases: vec![
            case!(vec![""], vec![]),
            case!(vec!["foo"], vec![(EventType::Text, Some("foo")),]),
            case!(
                vec![indoc! {"
                    foo
                    bar"}],
                vec![
                    (EventType::Text, Some("foo")),
                    (EventType::NewLine, None),
                    (EventType::Text, Some("bar")),
                ]
            ),
            case!(
                vec!["<`foo`>"],
                vec![(EventType::VerbatimEscaping, Some("foo")),]
            ),
            case!(
                vec!["foo<`bar`>",],
                vec![
                    (EventType::Text, Some("foo")),
                    (EventType::VerbatimEscaping, Some("bar")),
                ]
            ),
            case!(
                vec!["<`foo`>bar",],
                vec![
                    (EventType::VerbatimEscaping, Some("foo")),
                    (EventType::Text, Some("bar")),
                ]
            ),
            case!(
                vec!["<`foo`><`bar`>"],
                vec![
                    (EventType::VerbatimEscaping, Some("foo")),
                    (EventType::VerbatimEscaping, Some("bar")),
                ]
            ),
        ],
    }]
}
