#![cfg(test)]

mod support;

use support::{case, f, m};

use super::*;

use std::collections::HashSet;

use crate::{
    events::{Event, EventType},
    test_support::{self, report_failed_cases, FailedCase, GroupedCases},
};

#[test]
fn it_works() {
    use EventType::*;

    let table: Vec<GroupedCases<_>> = vec![
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
        .flat_map(|row| -> Vec<FailedCase> { row.collect_failed() })
        .collect();

    if failed_cases.is_empty() {
        return;
    }
    let faild_case_count = failed_cases.len();

    report_failed_cases(failed_cases);

    panic!("{} cases failed!", faild_case_count);
}
