use rotext_core::EventType;

use crate::suites::inline::support::{GroupedCases, case};

pub fn groups_wiki_link() -> Vec<GroupedCases> {
    vec![
        GroupedCases {
            group: "Wiki链接",
            cases: vec![
                case!(
                    vec!["[[页面]]", "[[␠页面]]", "[[页面␠]]", "[[␠页面␠]]",],
                    vec![
                        (EventType::EnterWikiLink, Some("页面")),
                        (EventType::Text, Some("页面")),
                        (EventType::ExitInline, None),
                    ]
                ),
                case!(
                    vec![
                        "[[<`页面`>]]",
                        "[[␠<`页面`>]]",
                        "[[<`页面`>␠]]",
                        "[[␠<`页面`>␠]]"
                    ],
                    vec![
                        (EventType::EnterWikiLink, Some("页面")),
                        (EventType::VerbatimEscaping, Some("页面")),
                        (EventType::ExitInline, None),
                    ]
                ),
                case!(vec!["foo[[页面]]bar",], vec![
                    (EventType::Text, Some("foo")),
                    (EventType::EnterWikiLink, Some("页面")),
                    (EventType::Text, Some("页面")),
                    (EventType::ExitInline, None),
                    (EventType::Text, Some("bar")),
                ]),
                case!(vec!["[[页面",], vec![(EventType::Text, Some("[[页面")),]),
                case!(vec!["[[a<`页面`>]]",], vec![
                    (EventType::Text, Some("[[a")),
                    (EventType::VerbatimEscaping, Some("页面")),
                    (EventType::Text, Some("]]")),
                ]),
                case!(vec!["[[a␣<`页面`>]]",], vec![
                    (EventType::Text, Some("[[a ")),
                    (EventType::VerbatimEscaping, Some("页面")),
                    (EventType::Text, Some("]]")),
                ]),
                case!(vec!["[[<`页面`>a]]",], vec![
                    (EventType::Text, Some("[[")),
                    (EventType::VerbatimEscaping, Some("页面")),
                    (EventType::Text, Some("a]]")),
                ]),
                case!(vec!["[[<`页面`>␣a]]",], vec![
                    (EventType::Text, Some("[[")),
                    (EventType::VerbatimEscaping, Some("页面")),
                    (EventType::Text, Some(" a]]")),
                ]),
            ],
        },
        GroupedCases {
            group: "Wiki链接>文本内容",
            cases: vec![
                case!(vec!["[[页面|内容]]",], vec![
                    (EventType::EnterWikiLink, Some("页面")),
                    (EventType::Text, Some("内容")),
                    (EventType::ExitInline, None),
                ]),
                case!(vec!["[[页面|␣内容]]",], vec![
                    (EventType::EnterWikiLink, Some("页面")),
                    (EventType::Text, Some(" 内容")),
                    (EventType::ExitInline, None),
                ]),
                case!(vec!["[[页面|内容␣]]",], vec![
                    (EventType::EnterWikiLink, Some("页面")),
                    (EventType::Text, Some("内容 ")),
                    (EventType::ExitInline, None),
                ]),
                case!(vec!["[[页面|内容\n]]",], vec![
                    (EventType::EnterWikiLink, Some("页面")),
                    (EventType::Text, Some("内容")),
                    (EventType::NewLine, None),
                    (EventType::ExitInline, None),
                ]),
                case!(vec!["[[页面|\n内容]]",], vec![
                    (EventType::EnterWikiLink, Some("页面")),
                    (EventType::NewLine, None),
                    (EventType::Text, Some("内容")),
                    (EventType::ExitInline, None),
                ]),
                case!(vec!["[[页面|内容\n第二行]]",], vec![
                    (EventType::EnterWikiLink, Some("页面")),
                    (EventType::Text, Some("内容")),
                    (EventType::NewLine, None),
                    (EventType::Text, Some("第二行")),
                    (EventType::ExitInline, None),
                ]),
                case!(vec!["[[页面|]]", "[[页面|", "[[页面|\n",], vec![
                    (EventType::EnterWikiLink, Some("页面")),
                    (EventType::ExitInline, None),
                ]),
                case!(vec!["[[页面|\n]]",], vec![
                    (EventType::EnterWikiLink, Some("页面")),
                    (EventType::NewLine, None),
                    (EventType::ExitInline, None),
                ]),
                case!(vec!["[[页面|]",], vec![
                    (EventType::EnterWikiLink, Some("页面")),
                    (EventType::Text, Some("]")),
                    (EventType::ExitInline, None),
                ]),
                case!(vec!["[[页面<`|`>内容]]",], vec![
                    (EventType::Text, Some("[[页面")),
                    (EventType::VerbatimEscaping, Some("|")),
                    (EventType::Text, Some("内容]]")),
                ]),
            ],
        },
        GroupedCases {
            group: "Wiki链接>其他行内内容",
            cases: vec![
                case!(vec!["[[页面|<`内容`>]]",], vec![
                    (EventType::EnterWikiLink, Some("页面")),
                    (EventType::VerbatimEscaping, Some("内容")),
                    (EventType::ExitInline, None),
                ]),
                case!(vec!["[[页面|a<`内容`>b]]",], vec![
                    (EventType::EnterWikiLink, Some("页面")),
                    (EventType::Text, Some("a")),
                    (EventType::VerbatimEscaping, Some("内容")),
                    (EventType::Text, Some("b")),
                    (EventType::ExitInline, None),
                ]),
                case!(vec!["[[页面|[*内容*]]]",], vec![
                    (EventType::EnterWikiLink, Some("页面")),
                    (EventType::EnterStrong, None),
                    (EventType::Text, Some("内容")),
                    (EventType::ExitInline, None),
                    (EventType::ExitInline, None),
                ]),
                case!(vec!["[[页面|a[*内容*]b]]",], vec![
                    (EventType::EnterWikiLink, Some("页面")),
                    (EventType::Text, Some("a")),
                    (EventType::EnterStrong, None),
                    (EventType::Text, Some("内容")),
                    (EventType::ExitInline, None),
                    (EventType::Text, Some("b")),
                    (EventType::ExitInline, None),
                ]),
                case!(vec!["[[页面|[[页面2]]]]", "[[页面|[[页面2]]",], vec![
                    (EventType::EnterWikiLink, Some("页面")),
                    (EventType::EnterWikiLink, Some("页面2")),
                    (EventType::Text, Some("页面2")),
                    (EventType::ExitInline, None),
                    (EventType::ExitInline, None),
                ]),
                case!(
                    vec!["[[页面|[[页面2|内容]]]]", "[[页面|[[页面2|内容]]",],
                    vec![
                        (EventType::EnterWikiLink, Some("页面")),
                        (EventType::EnterWikiLink, Some("页面2")),
                        (EventType::Text, Some("内容")),
                        (EventType::ExitInline, None),
                        (EventType::ExitInline, None),
                    ]
                ),
            ],
        },
    ]
}
