use indoc::indoc;

use crate::{
    events::EventType,
    test_suites::inline::support::{case, GroupedCases},
};

pub fn groups_basic() -> Vec<GroupedCases> {
    vec![
        GroupedCases {
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
        },
        GroupedCases {
            group: "基础>反斜杠转义",
            cases: vec![
                case!(
                    vec![r#"\"#, r#"\\"#],
                    vec![(EventType::Text, Some(r#"\"#)),]
                ),
                case!(
                    vec![r#"\\\"#],
                    vec![
                        (EventType::Text, Some(r#"\"#)),
                        (EventType::Text, Some(r#"\"#)),
                    ]
                ),
                case!(vec![r#"\x"#], vec![(EventType::Text, Some(r#"x"#)),]),
                case!(
                    vec![r#"\\\x"#],
                    vec![
                        (EventType::Text, Some(r#"\"#)),
                        (EventType::Text, Some(r#"x"#)),
                    ]
                ),
                case!(
                    vec![r#"\h\e\l\l\o"#],
                    vec![
                        (EventType::Text, Some(r#"h"#)),
                        (EventType::Text, Some(r#"e"#)),
                        (EventType::Text, Some(r#"l"#)),
                        (EventType::Text, Some(r#"l"#)),
                        (EventType::Text, Some(r#"o"#)),
                    ]
                ),
                case!(vec![r#"\🌍"#], vec![(EventType::Text, Some(r#"🌍"#)),]),
            ],
        },
        GroupedCases {
            group: "基础>硬换行标记",
            cases: vec![
                case!(
                    // 当前版本（1.7.1-nightly (7c2012d0 2024-07-26)）的 rustfmt
                    // 在遇到跨行文本最后是 “\” 时会自动缩进，但其他情况就不会，
                    // 因此导致这里和前面的缩进水平不一致… 后同。
                    vec![indoc! {r#"
                    foo\
                    bar"#}],
                    vec![
                        (EventType::Text, Some("foo")),
                        (EventType::NewLine, None),
                        (EventType::Text, Some("bar")),
                    ]
                ),
                case!(
                    vec![indoc! {r#"
                    \
                    bar"#}],
                    vec![(EventType::NewLine, None), (EventType::Text, Some("bar")),]
                ),
            ],
        },
        GroupedCases {
            group: "基础>行合并标记",
            cases: vec![
                case!(
                    vec![indoc! {r#"
                    foo_
                    bar"#}],
                    vec![
                        (EventType::Text, Some("foo")),
                        (EventType::Text, Some("bar")),
                    ]
                ),
                case!(
                    vec![indoc! {r#"
                    _
                    bar"#}],
                    vec![(EventType::Text, Some("bar")),]
                ),
            ],
        },
    ]
}
