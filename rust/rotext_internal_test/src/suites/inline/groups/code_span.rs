use indoc::indoc;

use rotext_core::EventType;

use crate::suites::inline::support::{case, GroupedCases};

pub fn groups_code_span() -> Vec<GroupedCases> {
    vec![GroupedCases {
        group: "行内代码",
        cases: vec![
            case!(
                vec!["[`foo`]", "[``foo``]", "[`␣foo␣`]", "[``␣foo␣``]", "[`foo"],
                vec![
                    (EventType::EnterCodeSpan, None),
                    (EventType::Text, Some("foo")),
                    (EventType::ExitInline, None),
                ]
            ),
            case!(
                vec!["[`\tfoo\t`]"],
                vec![
                    (EventType::EnterCodeSpan, None),
                    (EventType::Text, Some("\tfoo\t")),
                    (EventType::ExitInline, None),
                ]
            ),
            case!(
                vec!["[`foo`"],
                vec![
                    (EventType::EnterCodeSpan, None),
                    (EventType::Text, Some("foo`")),
                    (EventType::ExitInline, None),
                ]
            ),
            case!(
                vec!["[`foo␣<%%>"],
                vec![
                    (EventType::EnterCodeSpan, None),
                    (EventType::Text, Some("foo ")),
                    (EventType::ExitInline, None),
                ]
            ),
            case!(
                vec!["bar[`foo`]",],
                vec![
                    (EventType::Text, Some("bar")),
                    (EventType::EnterCodeSpan, None),
                    (EventType::Text, Some("foo")),
                    (EventType::ExitInline, None),
                ]
            ),
            case!(
                vec!["[`foo`]bar",],
                vec![
                    (EventType::EnterCodeSpan, None),
                    (EventType::Text, Some("foo")),
                    (EventType::ExitInline, None),
                    (EventType::Text, Some("bar")),
                ]
            ),
            case!(
                vec!["[``[`…`]``]"],
                vec![
                    (EventType::EnterCodeSpan, None),
                    (EventType::Text, Some("[`…`]")),
                    (EventType::ExitInline, None),
                ]
            ),
            case!(
                vec![
                    indoc! {"
                    [`foo
                    line␣2`]"},
                    indoc! {"
                    [`foo
                    line␣2"}
                ],
                vec![
                    (EventType::EnterCodeSpan, None),
                    (EventType::Text, Some("foo")),
                    (EventType::NewLine, None),
                    (EventType::Text, Some("line 2")),
                    (EventType::ExitInline, None),
                ]
            ),
        ],
    }]
}
