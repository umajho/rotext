#![cfg(test)]

use super::*;

use std::{collections::HashSet, panic::catch_unwind};

use crate::{
    events::{Event, EventType},
    test_utils::{report_failed_cases, FaildCase},
};

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
                    vec![(Unparsed, Some("Hello, world!"), None)]
                ),
                case!("<", vec![(Unparsed, Some("<"), None)]),
            ],
        },
        GroupedCases {
            group: "CRLF",
            cases: vec![
                case!("\r", vec![(NewLine, None, f!(">ln:2"))]),
                case!(
                    "\r\r",
                    vec![(NewLine, None, f!(">ln:2")), (NewLine, None, f!(">ln:3"))]
                ),
                case!("\n", vec![(NewLine, None, f!(">ln:2"))]),
                case!(
                    "\n\n",
                    vec![(NewLine, None, f!(">ln:2")), (NewLine, None, f!(">ln:3"))]
                ),
                case!("\r\n", vec![(NewLine, None, f!(">ln:2"))]),
                case!(
                    "Left\rRight",
                    vec![
                        (Unparsed, Some("Left"), None),
                        (NewLine, None, f!(">ln:2")),
                        (Unparsed, Some("Right"), None)
                    ]
                ),
                case!(
                    "Left\nRight",
                    vec![
                        (Unparsed, Some("Left"), None),
                        (NewLine, None, f!(">ln:2")),
                        (Unparsed, Some("Right"), None)
                    ]
                ),
                case!(
                    "Left\r\nRight",
                    vec![
                        (Unparsed, Some("Left"), None),
                        (NewLine, None, f!(">ln:2")),
                        (Unparsed, Some("Right"), None)
                    ]
                ),
            ],
        },
        GroupedCases {
            group: "逐字文本转义",
            cases: vec![
                case!("<` … `>", vec![(VerbatimEscaping, Some("…"), f!(">ln:1"))]),
                case!(
                    "<`` … `>",
                    vec![(VerbatimEscaping, Some("… `>"), f!("F", ">ln:1"))]
                ),
                case!(
                    "<` … ``>",
                    vec![(VerbatimEscaping, Some("… `"), f!(">ln:1"))]
                ),
                case!(
                    "<`` `> ``>",
                    vec![(VerbatimEscaping, Some("`>"), f!(">ln:1"))]
                ),
                case!(
                    "<` line 1\nline 2 `>",
                    vec![(VerbatimEscaping, Some("line 1\nline 2"), f!(">ln:2"))]
                ),
                case!(
                    "<` A `><` B `>",
                    vec![
                        (VerbatimEscaping, Some("A"), f!(">ln:1")),
                        (VerbatimEscaping, Some("B"), f!(">ln:1"))
                    ]
                ),
                case!(
                    "Left<` … `>Right",
                    vec![
                        (Unparsed, Some("Left"), None),
                        (VerbatimEscaping, Some("…"), f!(">ln:1")),
                        (Unparsed, Some("Right"), None)
                    ]
                ),
                case!(
                    "<` `> `>",
                    vec![
                        (VerbatimEscaping, Some(" "), f!(">ln:1")),
                        (Unparsed, Some(" `>"), None)
                    ]
                ),
                case!(
                    "<` <` `>",
                    vec![(VerbatimEscaping, Some("<`"), f!(">ln:1"))]
                ),
                case!(
                    "Foo<`Bar",
                    vec![
                        (Unparsed, Some("Foo"), None),
                        (VerbatimEscaping, Some("Bar"), f!("F", ">ln:1"))
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
        let parser = Parser::new(self.input.as_bytes(), NewParserOptions::default());
        let actual: Vec<_> = parser
            .map(|ev| -> EventMatcher {
                let ev: Event = ev.into();
                (
                    EventType::from(ev.discriminant()),
                    ev.content(self.input.as_bytes()),
                    ev.assertion_flags(),
                )
            })
            .collect();

        assert_eq!(self.expected, actual);
    }
}

type EventMatcher = (
    EventType,
    Option<&'static str>,
    Option<HashSet<&'static str>>,
);
