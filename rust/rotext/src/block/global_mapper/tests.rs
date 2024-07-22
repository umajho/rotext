#![cfg(test)]

use std::{time, vec};

use super::*;
use rstest::rstest;

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
    Mapped::CharAt(0), Mapped::LineFeed, Mapped::CharAt(2), Mapped::NextChar])]
// ### 空行
#[case("\n", vec![
    Mapped::LineFeed ])]
#[case("\r", vec![
    Mapped::LineFeed ])]
#[case("\r\n", vec![
    Mapped::LineFeed])]
#[case("\n\n", vec![
    Mapped::LineFeed, Mapped::LineFeed])]
#[case("\r\r", vec![
    Mapped::LineFeed, Mapped::LineFeed])]
#[case("\r\n\r\n", vec![
    Mapped::LineFeed, Mapped::LineFeed])]
#[case("a\n", vec![
    Mapped::CharAt(0), Mapped::LineFeed])]
#[case("a\n\n", vec![
    Mapped::CharAt(0), Mapped::LineFeed, Mapped::LineFeed])]
#[case("a\r\n\r\n", vec![
    Mapped::CharAt(0), Mapped::LineFeed, Mapped::LineFeed])]
// ### 一行开头的空格
#[case("  \n", vec![
    Mapped::CharAt(0), Mapped::NextChar, Mapped::LineFeed])]
#[case("  a", vec![
    Mapped::CharAt(0), Mapped::NextChar, Mapped::NextChar])]
#[case("a\n  \n", vec![
    Mapped::CharAt(0), Mapped::LineFeed, Mapped::CharAt(2), Mapped::NextChar,
    Mapped::LineFeed])]
#[case("  <` `>\n", vec![
    Mapped::CharAt(0), Mapped::NextChar,
    Mapped::Text(Range::new(4, 1)), Mapped::LineFeed])]
// ## 逐字文本转义转为文本
#[case("<`a`>", vec![
    Mapped::Text(Range::new(2, 1))])]
#[case("<`a\nb`>", vec![
    Mapped::Text(Range::new(2, 3))])]
#[case("<` a `>", vec![
    Mapped::Text(Range::new(3, 1))])]
#[case("<`  a  `>", vec![
    Mapped::Text(Range::new(3, 3))])]
#[case("<` `>", vec![
    Mapped::Text(Range::new(2, 1))])]
#[case("<`  `>", vec![
    Mapped::Text(Range::new(3, 0))])]
#[case("<`   `>", vec![
    Mapped::Text(Range::new(3, 1))])]
#[case("a<`` ` ``>bc", vec![
    Mapped::CharAt(0), Mapped::Text(Range::new(5, 1)),
    Mapped::CharAt(10), Mapped::NextChar])]
#[case("a<` b", vec![
    Mapped::CharAt(0), Mapped::Text(Range::new(4, 1))])]
#[case("a<` b ", vec![
    Mapped::CharAt(0), Mapped::Text(Range::new(4, 2))])]
#[case("a\n<`b`>", vec![
    Mapped::CharAt(0), Mapped::LineFeed, Mapped::Text(Range::new(4, 1))])]
#[case("a\n <`b`>", vec![
    Mapped::CharAt(0), Mapped::LineFeed, Mapped::CharAt(2),
    Mapped::Text(Range::new(5, 1))])]
#[case("<`b`>  c", vec![
    Mapped::Text(Range::new(2, 1)), Mapped::CharAt(5), Mapped::NextChar,
    Mapped::NextChar])]
#[timeout(time::Duration::from_secs(1))]
fn it_works(#[case] input: &str, #[case] expected: Vec<Mapped>) {
    let global_parser = global::Parser::new(input.as_bytes(), 0);
    let global_mapper = GlobalEventStreamMapper::new(input.as_bytes(), global_parser);

    let actual: Vec<_> = global_mapper.collect();

    assert_eq!(expected, actual);
}
