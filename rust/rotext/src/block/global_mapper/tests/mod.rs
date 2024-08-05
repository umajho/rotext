#![cfg(test)]

mod support;

use std::vec;

use support::{case, new_line, verbatim_escaping};

use crate::test_support::{self, report_panicked_cases, FailedCase, GroupedCases};

use super::*;

#[test]
fn it_works() {
    use Mapped::{CharAt, NextChar};

    let table: Vec<GroupedCases<_>> = vec![
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
                        verbatim_escaping!(4..5, 1),
                        new_line!(2)
                    ]
                ),
            ],
        },
        GroupedCases {
            group: "逐字文本转义转为文本",
            cases: vec![
                case!("<`a`>", vec![verbatim_escaping!(2..3, 1)]),
                case!("<`a\nb`>", vec![verbatim_escaping!(2..5, 2)]),
                case!("<` a `>", vec![verbatim_escaping!(3..4, 1)]),
                case!("<`  a  `>", vec![verbatim_escaping!(3..6, 1)]),
                case!("<` `>", vec![verbatim_escaping!(2..3, 1)]),
                case!("<`  `>", vec![verbatim_escaping!(3..3, 1)]),
                case!("<`   `>", vec![verbatim_escaping!(3..4, 1)]),
                case!(
                    "a<`` ` ``>bc",
                    vec![CharAt(0), verbatim_escaping!(5..6, 1), CharAt(10), NextChar]
                ),
                case!("a<` b", vec![CharAt(0), verbatim_escaping!(4..5, 1, "F")]),
                case!("a<` b ", vec![CharAt(0), verbatim_escaping!(4..6, 1, "F")]),
                case!(
                    "a\n<`b`>",
                    vec![CharAt(0), new_line!(2), verbatim_escaping!(4..5, 2)]
                ),
                case!(
                    "a\n <`b`>",
                    vec![
                        CharAt(0),
                        new_line!(2),
                        CharAt(2),
                        verbatim_escaping!(5..6, 2)
                    ]
                ),
                case!(
                    "<`b`>  c",
                    vec![verbatim_escaping!(2..3, 1), CharAt(5), NextChar, NextChar]
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

    report_panicked_cases(failed_cases);

    panic!("{} cases failed!", faild_case_count);
}
