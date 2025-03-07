use indoc::formatdoc;

use rotext_core::EventType;

use crate::suites::inline::support::{case, GroupedCases};

pub fn groups_strong_and_strikethrough() -> Vec<GroupedCases> {
    let mut result: Vec<GroupedCases> = vec![];

    for (name, s, enter_ev, another_s, another_enter_ev) in [
        (
            "加粗强调",
            '\'',
            EventType::EnterStrong,
            '~',
            EventType::EnterStrikethrough,
        ),
        (
            "删除线",
            '~',
            EventType::EnterStrikethrough,
            '\'',
            EventType::EnterStrong,
        ),
    ] {
        result.push(GroupedCases {
            group: name,
            cases: vec![
                case!(
                    vec![format!("[{s}foo{s}]").leak(), format!("[{s}foo").leak(),],
                    vec![
                        (enter_ev, None),
                        (EventType::Text, Some("foo")),
                        (EventType::ExitInline, None),
                    ]
                ),
                case!(
                    vec![format!("[{s} foo {s}]").leak()],
                    vec![
                        (enter_ev, None),
                        (EventType::Text, Some(" foo ")),
                        (EventType::ExitInline, None),
                    ]
                ),
                case!(
                    vec![format!("[{s}foo{s}").leak()],
                    vec![
                        (enter_ev, None),
                        (EventType::Text, Some(format!("foo{s}").leak())),
                        (EventType::ExitInline, None),
                    ]
                ),
                case!(
                    vec![format!("bar[{s}foo{s}]").leak()],
                    vec![
                        (EventType::Text, Some("bar")),
                        (enter_ev, None),
                        (EventType::Text, Some("foo")),
                        (EventType::ExitInline, None),
                    ]
                ),
                case!(
                    vec![format!("[{s}foo{s}]bar").leak()],
                    vec![
                        (enter_ev, None),
                        (EventType::Text, Some("foo")),
                        (EventType::ExitInline, None),
                        (EventType::Text, Some("bar")),
                    ]
                ),
                case!(
                    vec![
                        formatdoc! {"
                        [{s}foo
                        line 2{s}]"}
                        .leak(),
                        formatdoc! {"
                        [{s}foo
                        line 2"}
                        .leak(),
                    ],
                    vec![
                        (enter_ev, None),
                        (EventType::Text, Some("foo")),
                        (EventType::NewLine, None),
                        (EventType::Text, Some("line 2")),
                        (EventType::ExitInline, None),
                    ]
                ),
                case!(
                    vec![formatdoc! {"
                        [{s}
                        foo
                        {s}]"}
                    .leak()],
                    vec![
                        (enter_ev, None),
                        (EventType::NewLine, None),
                        (EventType::Text, Some("foo")),
                        (EventType::NewLine, None),
                        (EventType::ExitInline, None),
                    ]
                ),
            ],
        });
        result.push(GroupedCases {
            group: format!("{name}>嵌套").leak(),
            cases: vec![
                case!(
                    vec![formatdoc! {"
                        [{s}[{s}1{s}] [{another_s}
                        2 [{s}3{s}]{another_s}] >>TP.4
                        [=d5]{s}]"}
                    .leak()],
                    vec![
                        (enter_ev, None),
                        (enter_ev, None),
                        (EventType::Text, Some("1")),
                        (EventType::ExitInline, None),
                        (EventType::Text, Some(" ")),
                        (another_enter_ev, None),
                        (EventType::NewLine, None),
                        (EventType::Text, Some("2 ")),
                        (enter_ev, None),
                        (EventType::Text, Some("3")),
                        (EventType::ExitInline, None),
                        (EventType::ExitInline, None),
                        (EventType::Text, Some(" ")),
                        (EventType::RefLink, Some("TP.4")),
                        (EventType::NewLine, None),
                        (EventType::Dicexp, Some("d5")),
                        (EventType::ExitInline, None),
                    ]
                ),
                case!(
                    vec![format!("[{s}[{another_s}foo").leak()],
                    vec![
                        (enter_ev, None),
                        (another_enter_ev, None),
                        (EventType::Text, Some("foo")),
                        (EventType::ExitInline, None),
                        (EventType::ExitInline, None),
                    ]
                ),
                case!(
                    vec![format!("[{s}[{another_s}foo{s}]").leak()],
                    vec![
                        (enter_ev, None),
                        (another_enter_ev, None),
                        (EventType::Text, Some("foo")),
                        (EventType::ExitInline, None),
                        (EventType::ExitInline, None),
                    ]
                ),
            ],
        });
    }

    result
}
