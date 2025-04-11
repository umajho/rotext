use rotext_core::EventType;

use crate::suites::inline::support::{GroupedCases, case};

pub fn groups_call() -> Vec<GroupedCases> {
    vec![
        GroupedCases {
            group: "调用>不匹配",
            cases: vec![
                case!(vec!["[{"], vec![(EventType::Text, Some("[{")),]),
                case!(vec!["[{}]"], vec![(EventType::Text, Some("[{}]")),]),
                case!(vec!["[{␣}]"], vec![(EventType::Text, Some("[{ }]")),]),
                case!(vec!["[{\nfoo\n}]"], vec![
                    (EventType::Text, Some("[{")),
                    (EventType::NewLine, None),
                    (EventType::Text, Some("foo")),
                    (EventType::NewLine, None),
                    (EventType::Text, Some("}]")),
                ]),
                case!(vec!["[{<%C%>foo<%C%>}]"], vec![
                    (EventType::Text, Some("[{")),
                    (EventType::Text, Some("foo")),
                    (EventType::Text, Some("}]")),
                ]),
                case!(vec!["[{foo"], vec![(EventType::Text, Some("[{foo")),]),
                case!(vec!["[{#␣foo}]"], vec![(
                    EventType::Text,
                    Some("[{# foo}]")
                ),]),
                case!(vec!["[{foo["], vec![(EventType::Text, Some("[{foo[")),]),
            ],
        },
        GroupedCases {
            group: "调用>无参数",
            cases: vec![
                case!(vec!["[{foo}]", "[{␠foo␠}]",], vec![
                    (EventType::EnterCallOnTemplate, Some("foo")),
                    (EventType::ExitInline, None)
                ]),
                case!(vec!["[{foo␣bar}]",], vec![
                    (EventType::EnterCallOnTemplate, Some("foo bar")),
                    (EventType::ExitInline, None)
                ]),
                case!(vec!["[{<`*`>}]", "[{␠<`*`>␠}]",], vec![
                    (EventType::EnterCallOnTemplate, Some("*")),
                    (EventType::ExitInline, None)
                ]),
                case!(vec!["[{#foo}]", "[{␠#foo␠}]",], vec![
                    (EventType::EnterCallOnExtension, Some("foo")),
                    (EventType::ExitInline, None)
                ]),
                case!(vec!["[{#<`*`>}]",], vec![
                    (EventType::EnterCallOnExtension, Some("*")),
                    (EventType::ExitInline, None)
                ]),
            ],
        },
    ]
}
