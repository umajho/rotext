#![cfg(test)]

use std::{panic::catch_unwind, vec};

use crate::test_utils::{report_failed_cases, FaildCase};

use super::*;

macro_rules! new_line {
    ($line_number_after:expr) => {
        Mapped::NewLine(NewLine {
            #[cfg(feature = "line-number")]
            line_number_after: $line_number_after,
        })
    };
}
macro_rules! verbatim_escaping {
    (($content_start:expr, $content_length:expr), $line_number_after:expr) => {
        Mapped::VerbatimEscaping(VerbatimEscaping {
            content: Range::new($content_start, $content_length),
            is_closed_forcedly: false,
            #[cfg(feature = "line-number")]
            line_number_after: $line_number_after,
        })
    };
    (($content_start:expr, $content_length:expr), $line_number_after:expr, "F") => {
        Mapped::VerbatimEscaping(VerbatimEscaping {
            content: Range::new($content_start, $content_length),
            is_closed_forcedly: true,
            #[cfg(feature = "line-number")]
            line_number_after: $line_number_after,
        })
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
    use Mapped::{CharAt, NextChar};

    let table: Vec<GroupedCases> = vec![
        GroupedCases {
            group: "无特殊语法",
            cases: vec![
                case!("", vec![]),
                case!("  ", vec![CharAt(0), NextChar]),
                case!("a", vec![CharAt(0)]),
                case!("ab", vec![CharAt(0), NextChar]),
            ],
        },
        GroupedCases {
            group: "换行",
            cases: vec![case!(
                "a\nbc",
                vec![CharAt(0), new_line!(2), CharAt(2), NextChar]
            )],
        },
        GroupedCases {
            group: "换行>空行",
            cases: vec![
                case!("\n", vec![new_line!(2)]),
                case!("\r", vec![new_line!(2)]),
                case!("\r\n", vec![new_line!(2)]),
                case!("\n\n", vec![new_line!(2), new_line!(3)]),
                case!("\r\r", vec![new_line!(2), new_line!(3)]),
                case!("\r\n\r\n", vec![new_line!(2), new_line!(3)]),
                case!("a\n", vec![CharAt(0), new_line!(2)]),
                case!("a\n\n", vec![CharAt(0), new_line!(2), new_line!(3)]),
                case!("a\r\n\r\n", vec![CharAt(0), new_line!(2), new_line!(3)]),
            ],
        },
        GroupedCases {
            group: "换行>一行开头的空格",
            cases: vec![
                case!("  \n", vec![CharAt(0), NextChar, new_line!(2)]),
                case!("  a", vec![CharAt(0), NextChar, NextChar]),
                case!(
                    "a\n  \n",
                    vec![CharAt(0), new_line!(2), CharAt(2), NextChar, new_line!(3)]
                ),
                case!(
                    "  <` `>\n",
                    vec![
                        CharAt(0),
                        NextChar,
                        verbatim_escaping!((4, 1), 1),
                        new_line!(2)
                    ]
                ),
            ],
        },
        GroupedCases {
            group: "逐字文本转义转为文本",
            cases: vec![
                case!("<`a`>", vec![verbatim_escaping!((2, 1), 1)]),
                case!("<`a\nb`>", vec![verbatim_escaping!((2, 3), 2)]),
                case!("<` a `>", vec![verbatim_escaping!((3, 1), 1)]),
                case!("<`  a  `>", vec![verbatim_escaping!((3, 3), 1)]),
                case!("<` `>", vec![verbatim_escaping!((2, 1), 1)]),
                case!("<`  `>", vec![verbatim_escaping!((3, 0), 1)]),
                case!("<`   `>", vec![verbatim_escaping!((3, 1), 1)]),
                case!(
                    "a<`` ` ``>bc",
                    vec![
                        CharAt(0),
                        verbatim_escaping!((5, 1), 1),
                        CharAt(10),
                        NextChar
                    ]
                ),
                case!("a<` b", vec![CharAt(0), verbatim_escaping!((4, 1), 1, "F")]),
                case!(
                    "a<` b ",
                    vec![CharAt(0), verbatim_escaping!((4, 2), 1, "F")]
                ),
                case!(
                    "a\n<`b`>",
                    vec![CharAt(0), new_line!(2), verbatim_escaping!((4, 1), 2)]
                ),
                case!(
                    "a\n <`b`>",
                    vec![
                        CharAt(0),
                        new_line!(2),
                        CharAt(2),
                        verbatim_escaping!((5, 1), 2)
                    ]
                ),
                case!(
                    "<`b`>  c",
                    vec![verbatim_escaping!((2, 1), 1), CharAt(5), NextChar, NextChar]
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
                    nth_case_in_group: i + 1,
                    nth_case_variant_in_case: None,
                    auto_variant: None,
                    input: case.input.to_string(),
                    panic,
                })
            })
            .collect()
    }
}

struct Case {
    input: &'static str,
    expected: Vec<Mapped>,
}
impl Case {
    fn assert_ok(&self) {
        let global_parser =
            global::Parser::new(self.input.as_bytes(), global::NewParserOptions::default());
        let global_mapper = GlobalEventStreamMapper::new(self.input.as_bytes(), global_parser);

        let actual: Vec<_> = global_mapper.collect();

        assert_eq!(self.expected, actual);
    }
}
