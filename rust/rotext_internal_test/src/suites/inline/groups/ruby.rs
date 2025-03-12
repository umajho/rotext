use rotext_core::EventType;

use crate::suites::inline::support::{case, GroupedCases};

pub fn groups_ruby() -> Vec<GroupedCases> {
    vec![GroupedCases {
        group: "注音",
        cases: vec![
            case!(
                vec!["[;foo]",],
                vec![
                    (EventType::EnterRuby, None),
                    (EventType::Text, Some("foo")),
                    (EventType::ExitInline, None),
                ]
            ),
            case!(
                vec!["[;foo:bar]", "[;foo␠:␠bar]",],
                vec![
                    (EventType::EnterRuby, None),
                    (EventType::Text, Some("foo")),
                    (EventType::EnterRubyText, None),
                    (EventType::Text, Some("bar")),
                    (EventType::ExitInline, None),
                    (EventType::ExitInline, None),
                ]
            ),
            case!(
                vec!["[;␣foo␠:␠bar␣]",],
                vec![
                    (EventType::EnterRuby, None),
                    (EventType::Text, Some(" foo")),
                    (EventType::EnterRubyText, None),
                    (EventType::Text, Some("bar ")),
                    (EventType::ExitInline, None),
                    (EventType::ExitInline, None),
                ]
            ),
        ],
    }]
}
