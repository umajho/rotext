#![cfg(test)]

use super::*;
use rstest::rstest;

use std::{collections::HashSet, time};

use crate::events::{Event, EventType};

type EventCase<'a> = (EventType, Option<&'a str>, Option<HashSet<&'a str>>);

macro_rules! f {
    ($($flag:expr),*) => {
        Some(HashSet::from([$($flag),*]))
    };
}

#[rstest]
// ## 无特殊语法
#[case("", vec![])]
#[case("Hello, world!", vec![
    (EventType::Unparsed, Some("Hello, world!"), None)])]
#[case("<", vec![
    (EventType::Unparsed, Some("<"), None)])]
// ## CRLF
#[case("\r", vec![
    (EventType::NewLine, None, f!(">ln:2"))])]
#[case("\r\r", vec![
    (EventType::NewLine, None, f!(">ln:2")),
    (EventType::NewLine, None, f!(">ln:3"))])]
#[case("\n", vec![
    (EventType::NewLine, None, f!(">ln:2"))])]
#[case("\n\n", vec![
    (EventType::NewLine, None, f!(">ln:2")),
    (EventType::NewLine, None, f!(">ln:3"))])]
#[case("\r\n", vec![
    (EventType::NewLine, None, f!(">ln:2"))])]
#[case("Left\rRight", vec![
    (EventType::Unparsed, Some("Left"), None),
    (EventType::NewLine, None, f!(">ln:2")),
    (EventType::Unparsed, Some("Right"), None)])]
#[case("Left\nRight", vec![
    (EventType::Unparsed, Some("Left"), None),
    (EventType::NewLine, None, f!(">ln:2")),
    (EventType::Unparsed, Some("Right"), None)])]
#[case("Left\r\nRight", vec![
    (EventType::Unparsed, Some("Left"), None),
    (EventType::NewLine, None, f!(">ln:2")),
    (EventType::Unparsed, Some("Right"), None)])]
// ## 逐字文本转义
#[case("<` … `>", vec![
    (EventType::VerbatimEscaping, Some("…"), f!(">ln:1"))])]
#[case("<`` … `>", vec![
    (EventType::VerbatimEscaping, Some("… `>"), f!("F", ">ln:1"))])]
#[case("<` … ``>", vec![
    (EventType::VerbatimEscaping, Some("… `"), f!(">ln:1"))])]
#[case("<`` `> ``>", vec![
    (EventType::VerbatimEscaping, Some("`>"), f!(">ln:1"))])]
#[case("<` line 1\nline 2 `>", vec![
    (EventType::VerbatimEscaping, Some("line 1\nline 2"), f!(">ln:2"))])]
#[case("<` A `><` B `>", vec![
    (EventType::VerbatimEscaping, Some("A"), f!(">ln:1")),
    (EventType::VerbatimEscaping, Some("B"), f!(">ln:1"))])]
#[case("Left<` … `>Right", vec![
    (EventType::Unparsed, Some("Left"), None),
    (EventType::VerbatimEscaping, Some("…"), f!(">ln:1")),
    (EventType::Unparsed, Some("Right"), None)])]
#[case("<` `> `>", vec![
    (EventType::VerbatimEscaping, Some(" "), f!(">ln:1")),
    (EventType::Unparsed, Some(" `>"), None)])]
#[case("<` <` `>", vec![
    (EventType::VerbatimEscaping, Some("<`"), f!(">ln:1"))])]
#[case("Foo<`Bar", vec![
    (EventType::Unparsed, Some("Foo"), None),
    (EventType::VerbatimEscaping, Some("Bar"), f!("F", ">ln:1"))])]
#[timeout(time::Duration::from_secs(1))]
fn it_works(#[case] input: &str, #[case] expected: Vec<EventCase>) {
    let parser = Parser::new(input.as_bytes(), NewParserOptions::default());
    let actual: Vec<_> = parser
        .map(|ev| -> EventCase {
            let ev: Event = ev.into();
            (
                EventType::from(ev.discriminant()),
                ev.content(input.as_bytes()),
                ev.assertion_flags(),
            )
        })
        .collect();

    assert_eq!(expected, actual);
}
