#![cfg(test)]

use std::{time, vec};

use super::*;
use rstest::rstest;

macro_rules! new_line {
    ($line_number_after:expr) => {
        Mapped::NewLine(NewLine {
            line_number_after: $line_number_after,
        })
    };
}
macro_rules! verbatim_escaping {
    (($content_start:expr, $content_length:expr), $line_number_after:expr) => {
        Mapped::VerbatimEscaping(VerbatimEscaping {
            content: Range::new($content_start, $content_length),
            is_closed_forcedly: false,
            line_number_after: $line_number_after,
        })
    };
    (($content_start:expr, $content_length:expr), $line_number_after:expr, "F") => {
        Mapped::VerbatimEscaping(VerbatimEscaping {
            content: Range::new($content_start, $content_length),
            is_closed_forcedly: true,
            line_number_after: $line_number_after,
        })
    };
}

#[rstest]
// ## 无特殊语法
#[case("", vec![])]
#[case("  ", vec![
    Mapped::CharAt(0), Mapped::NextChar])]
#[case("a", vec![
    Mapped::CharAt(0)])]
#[case("ab", vec![
    Mapped::CharAt(0), Mapped::NextChar])]
// ## 换行
#[case("a\nbc", vec![
    Mapped::CharAt(0), new_line!(2), Mapped::CharAt(2), Mapped::NextChar])]
// ### 空行
#[case("\n", vec![
    new_line!(2)])]
#[case("\r", vec![
    new_line!(2)])]
#[case("\r\n", vec![
    new_line!(2)])]
#[case("\n\n", vec![
    new_line!(2), new_line!(3)])]
#[case("\r\r", vec![
    new_line!(2), new_line!(3)])]
#[case("\r\n\r\n", vec![
    new_line!(2), new_line!(3)])]
#[case("a\n", vec![
    Mapped::CharAt(0), new_line!(2)])]
#[case("a\n\n", vec![
    Mapped::CharAt(0), new_line!(2), new_line!(3)])]
#[case("a\r\n\r\n", vec![
    Mapped::CharAt(0), new_line!(2), new_line!(3)])]
// ### 一行开头的空格
#[case("  \n", vec![
    Mapped::CharAt(0), Mapped::NextChar, new_line!(2)])]
#[case("  a", vec![
    Mapped::CharAt(0), Mapped::NextChar, Mapped::NextChar])]
#[case("a\n  \n", vec![
    Mapped::CharAt(0), new_line!(2), Mapped::CharAt(2), Mapped::NextChar,
    new_line!(3)])]
#[case("  <` `>\n", vec![
    Mapped::CharAt(0), Mapped::NextChar,
    verbatim_escaping!((4, 1), 1), new_line!(2)])]
// ## 逐字文本转义转为文本
#[case("<`a`>", vec![
    verbatim_escaping!((2, 1), 1)])]
#[case("<`a\nb`>", vec![
    verbatim_escaping!((2, 3), 2)])]
#[case("<` a `>", vec![
    verbatim_escaping!((3, 1), 1)])]
#[case("<`  a  `>", vec![
    verbatim_escaping!((3, 3), 1)])]
#[case("<` `>", vec![
    verbatim_escaping!((2, 1), 1)])]
#[case("<`  `>", vec![
    verbatim_escaping!((3, 0), 1)])]
#[case("<`   `>", vec![
    verbatim_escaping!((3, 1), 1)])]
#[case("a<`` ` ``>bc", vec![
    Mapped::CharAt(0), verbatim_escaping!((5, 1), 1),
    Mapped::CharAt(10), Mapped::NextChar])]
#[case("a<` b", vec![
    Mapped::CharAt(0), verbatim_escaping!((4, 1), 1, "F")])]
#[case("a<` b ", vec![
    Mapped::CharAt(0), verbatim_escaping!((4, 2), 1, "F")])]
#[case("a\n<`b`>", vec![
    Mapped::CharAt(0), new_line!(2), verbatim_escaping!((4, 1), 2)])]
#[case("a\n <`b`>", vec![
    Mapped::CharAt(0), new_line!(2), Mapped::CharAt(2),
    verbatim_escaping!((5, 1), 2)])]
#[case("<`b`>  c", vec![
    verbatim_escaping!((2, 1), 1), Mapped::CharAt(5), Mapped::NextChar,
    Mapped::NextChar])]
#[timeout(time::Duration::from_secs(1))]
fn it_works(#[case] input: &str, #[case] expected: Vec<Mapped>) {
    let global_parser = global::Parser::new(input.as_bytes(), global::NewParserOptions::default());
    let global_mapper = GlobalEventStreamMapper::new(input.as_bytes(), global_parser);

    let actual: Vec<_> = global_mapper.collect();

    assert_eq!(expected, actual);
}
