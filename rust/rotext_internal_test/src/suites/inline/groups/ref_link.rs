use rotext_core::EventType;

use crate::suites::inline::support::{GroupedCases, case};

pub fn groups_ref_link() -> Vec<GroupedCases> {
    vec![
        GroupedCases {
            group: "引用链接",
            cases: vec![
                case!(vec![">>", ">>␠"], vec![(EventType::Text, Some(">>")),]),
                case!(vec![">>TP",], vec![(EventType::Text, Some(">>TP")),]),
                case!(vec![">>TP.",], vec![(EventType::Text, Some(">>TP.")),]),
                case!(vec![">>abc",], vec![(EventType::Text, Some(">>abc")),]),
                case!(vec![">>42",], vec![(EventType::Text, Some(">>42")),]),
                case!(vec![">>TP.x"], vec![(EventType::RefLink, Some("TP.x")),]),
                case!(vec![">>No.x"], vec![(EventType::RefLink, Some("No.x")),]),
                case!(vec!["abc>>TP.x"], vec![
                    (EventType::Text, Some("abc")),
                    (EventType::RefLink, Some("TP.x")),
                ]),
                case!(vec![">>>TP.x"], vec![
                    (EventType::Text, Some(">")),
                    (EventType::RefLink, Some("TP.x")),
                ]),
                case!(vec![">>>>TP.x"], vec![
                    (EventType::Text, Some(">>")),
                    (EventType::RefLink, Some("TP.x")),
                ]),
            ],
        },
        GroupedCases {
            group: "引用链接>绝对目标>串号",
            cases: vec![
                case!(vec![">>TP.abc", ">>TP.abc␠"], vec![(
                    EventType::RefLink,
                    Some("TP.abc")
                ),]),
                case!(vec![">>TP.abc1",], vec![
                    (EventType::RefLink, Some("TP.abc")),
                    (EventType::Text, Some("1"))
                ]),
                case!(vec![">>TP.abc?",], vec![
                    (EventType::RefLink, Some("TP.abc")),
                    (EventType::Text, Some("?"))
                ]),
                case!(vec![">>TP.abc#",], vec![
                    (EventType::RefLink, Some("TP.abc")),
                    (EventType::Text, Some("#"))
                ]),
                case!(vec![">>TP.abc#a",], vec![
                    (EventType::RefLink, Some("TP.abc")),
                    (EventType::Text, Some("#a"))
                ]),
                case!(vec![">>TP.abc#?",], vec![
                    (EventType::RefLink, Some("TP.abc")),
                    (EventType::Text, Some("#?"))
                ]),
            ],
        },
        GroupedCases {
            group: "引用链接>绝对目标>串号与楼层号",
            cases: vec![
                case!(vec![">>TP.abc#123", ">>TP.abc#123␠"], vec![(
                    EventType::RefLink,
                    Some("TP.abc#123")
                ),]),
                case!(vec![">>TP.abc#123a",], vec![
                    (EventType::RefLink, Some("TP.abc#123")),
                    (EventType::Text, Some("a"))
                ]),
                case!(vec![">>TP.abc#123?",], vec![
                    (EventType::RefLink, Some("TP.abc#123")),
                    (EventType::Text, Some("?"))
                ]),
            ],
        },
        GroupedCases {
            group: "引用链接>绝对目标>贴号",
            cases: vec![
                case!(vec![">>TP.456", ">>TP.456␠"], vec![(
                    EventType::RefLink,
                    Some("TP.456")
                ),]),
                case!(vec![">>TP.456a",], vec![
                    (EventType::RefLink, Some("TP.456")),
                    (EventType::Text, Some("a"))
                ]),
                case!(vec![">>TP.456?",], vec![
                    (EventType::RefLink, Some("TP.456")),
                    (EventType::Text, Some("?"))
                ]),
            ],
        },
        GroupedCases {
            group: "引用链接>串内楼层目标",
            cases: vec![
                case!(vec![">>#123", ">>#123␠"], vec![(
                    EventType::RefLink,
                    Some("#123")
                ),]),
                case!(vec![">>#123a",], vec![
                    (EventType::RefLink, Some("#123")),
                    (EventType::Text, Some("a"))
                ]),
                case!(vec![">>#123?",], vec![
                    (EventType::RefLink, Some("#123")),
                    (EventType::Text, Some("?"))
                ]),
            ],
        },
    ]
}

// case!(
//     vec![">>TP.123abc"],
//     vec![
//         (EventType::RefLink, Some("TP.123")),
//         (EventType::Text, Some("abc")),
//     ]
// ),
