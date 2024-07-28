#![cfg(test)]

use super::*;

use std::{collections::HashSet, panic::catch_unwind};

use crate::{
    events::{Event, EventType},
    test_utils::{report_failed_cases, FaildCase},
};

/// event matcher
macro_rules! m {
    ($event_type:expr, $content:expr, $flags:expr) => {
        EventMatcher {
            event_type: $event_type,
            content: $content,
            flags: $flags,
        }
    };
}

/// flag
macro_rules! f {
    ($($flag:expr),*) => {
        Some(HashSet::from([$($flag),*]))
    };
}

macro_rules! case {
    ($input:expr, $expected:expr) => {
        Case {
            input: $input,
            expected: $expected,
        }
    };
}

#[test]
fn it_works() {
    use EventType::*;

    let table: Vec<GroupedCases> = vec![
        GroupedCases {
            group: "无特殊语法",
            cases: vec![
                case!("", vec![]),
                case!(
                    "Hello, world!",
                    vec![m!(Unparsed, Some("Hello, world!"), None)]
                ),
                case!("<", vec![m!(Unparsed, Some("<"), None)]),
            ],
        },
        GroupedCases {
            group: "CRLF",
            cases: vec![
                case!("\r", vec![m!(NewLine, None, f!(">ln:2"))]),
                case!(
                    "\r\r",
                    vec![
                        m!(NewLine, None, f!(">ln:2")),
                        m!(NewLine, None, f!(">ln:3"))
                    ]
                ),
                case!("\n", vec![m!(NewLine, None, f!(">ln:2"))]),
                case!(
                    "\n\n",
                    vec![
                        m!(NewLine, None, f!(">ln:2")),
                        m!(NewLine, None, f!(">ln:3"))
                    ]
                ),
                case!("\r\n", vec![m!(NewLine, None, f!(">ln:2"))]),
                case!(
                    "Left\rRight",
                    vec![
                        m!(Unparsed, Some("Left"), None),
                        m!(NewLine, None, f!(">ln:2")),
                        m!(Unparsed, Some("Right"), None)
                    ]
                ),
                case!(
                    "Left\nRight",
                    vec![
                        m!(Unparsed, Some("Left"), None),
                        m!(NewLine, None, f!(">ln:2")),
                        m!(Unparsed, Some("Right"), None)
                    ]
                ),
                case!(
                    "Left\r\nRight",
                    vec![
                        m!(Unparsed, Some("Left"), None),
                        m!(NewLine, None, f!(">ln:2")),
                        m!(Unparsed, Some("Right"), None)
                    ]
                ),
            ],
        },
        GroupedCases {
            group: "逐字文本转义",
            cases: vec![
                case!(
                    "<` … `>",
                    vec![m!(VerbatimEscaping, Some("…"), f!(">ln:1"))]
                ),
                case!(
                    "<`` … `>",
                    vec![m!(VerbatimEscaping, Some("… `>"), f!("F", ">ln:1"))]
                ),
                case!(
                    "<` … ``>",
                    vec![m!(VerbatimEscaping, Some("… `"), f!(">ln:1"))]
                ),
                case!(
                    "<`` `> ``>",
                    vec![m!(VerbatimEscaping, Some("`>"), f!(">ln:1"))]
                ),
                case!(
                    "<` line 1\nline 2 `>",
                    vec![m!(VerbatimEscaping, Some("line 1\nline 2"), f!(">ln:2"))]
                ),
                case!(
                    "<` A `><` B `>",
                    vec![
                        m!(VerbatimEscaping, Some("A"), f!(">ln:1")),
                        m!(VerbatimEscaping, Some("B"), f!(">ln:1"))
                    ]
                ),
                case!(
                    "Left<` … `>Right",
                    vec![
                        m!(Unparsed, Some("Left"), None),
                        m!(VerbatimEscaping, Some("…"), f!(">ln:1")),
                        m!(Unparsed, Some("Right"), None)
                    ]
                ),
                case!(
                    "<` `> `>",
                    vec![
                        m!(VerbatimEscaping, Some(" "), f!(">ln:1")),
                        m!(Unparsed, Some(" `>"), None)
                    ]
                ),
                case!(
                    "<` <` `>",
                    vec![m!(VerbatimEscaping, Some("<`"), f!(">ln:1"))]
                ),
                case!(
                    "Foo<`Bar",
                    vec![
                        m!(Unparsed, Some("Foo"), None),
                        m!(VerbatimEscaping, Some("Bar"), f!("F", ">ln:1"))
                    ]
                ),
            ],
        },
    ];

    let failed_cases: Vec<_> = table
        .iter()
        .flat_map(|row| -> Vec<FaildCase> { row.collect_failed() })
        .collect();

    if failed_cases.is_empty() {
        return;
    }
    let faild_case_count = failed_cases.len();

    report_failed_cases(failed_cases);

    panic!("{} cases failed!", faild_case_count);
}

struct GroupedCases {
    group: &'static str,
    cases: Vec<Case>,
}
impl GroupedCases {
    fn collect_failed(&self) -> Vec<FaildCase> {
        self.cases
            .iter()
            .enumerate()
            .filter_map(|(i, case)| -> Option<FaildCase> {
                let panic = catch_unwind(|| case.assert_ok()).err()?;
                Some(FaildCase {
                    group: self.group,
                    auto_variant: None,
                    nth_case_in_group: i + 1,
                    nth_case_variant_in_case: None,
                    input: case.input.to_string(),
                    panic,
                })
            })
            .collect()
    }
}

struct Case {
    input: &'static str,
    expected: Vec<EventMatcher>,
}
impl Case {
    fn assert_ok(&self) {
        let expected: Vec<EventMatcher> = if cfg!(not(feature = "line-number")) {
            self.expected
                .clone()
                .into_iter()
                .map(|mut m| {
                    if let Some(flags) = m.flags {
                        let new_flags: HashSet<&str> = flags
                            .into_iter()
                            .filter(|f| !f.starts_with(">ln:"))
                            .collect();
                        m.flags = if new_flags.is_empty() {
                            None
                        } else {
                            Some(new_flags)
                        };
                    }
                    m
                })
                .collect()
        } else {
            self.expected.clone()
        };

        let parser = Parser::new(self.input.as_bytes(), NewParserOptions::default());
        let actual: Vec<_> = parser
            .map(|ev| -> EventMatcher {
                let ev: Event = ev.into();
                EventMatcher {
                    event_type: EventType::from(ev.discriminant()),
                    content: ev.content(self.input.as_bytes()),
                    flags: ev.assertion_flags(),
                }
            })
            .collect();

        assert_eq!(expected, actual);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct EventMatcher {
    event_type: EventType,
    content: Option<&'static str>,
    flags: Option<HashSet<&'static str>>,
}
