#![cfg(test)]

use super::*;
use rstest::rstest;

use std::{collections::HashSet, time};

use crate::events::{Event, EventType};

type EventCase<'a> = (EventType, Option<&'a str>, Option<HashSet<&'a str>>);

#[rstest]
// ## 无特殊语法
#[case("", vec![])]
#[case("Hello, world!", vec![
    (EventType::Unparsed, Some("Hello, world!"), None)])]
#[case("<", vec![
    (EventType::Unparsed, Some("<"), None)])]
// ### CR
#[case("\r", vec![
    (EventType::CarriageReturn, None, None)])]
#[case("Left\rRight", vec![
    (EventType::Unparsed, Some("Left"), None),
    (EventType::CarriageReturn, None, None),
    (EventType::Unparsed, Some("Right"), None)])]
// ## 逐字文本转义
#[case("<` … `>", vec![
    (EventType::VerbatimEscaping, Some(" … "), None)])]
#[case("<`` … `>", vec![
    (EventType::VerbatimEscaping, Some(" … `>"), Some(HashSet::from(["F"])))])]
#[case("<` … ``>", vec![
    (EventType::VerbatimEscaping, Some(" … `"), None)])]
#[case("<`` `> ``>", vec![
    (EventType::VerbatimEscaping, Some(" `> "), None)])]
#[case("<` line 1\nline 2 `>", vec![
    (EventType::VerbatimEscaping, Some(" line 1\nline 2 "), None)])]
#[case("<` A `><` B `>", vec![
    (EventType::VerbatimEscaping, Some(" A "), None),
    (EventType::VerbatimEscaping, Some(" B "), None)])]
#[case("Left<` … `>Right", vec![
    (EventType::Unparsed, Some("Left"), None),
    (EventType::VerbatimEscaping, Some(" … "), None),
    (EventType::Unparsed, Some("Right"), None)])]
#[case("<` `> `>", vec![
    (EventType::VerbatimEscaping, Some(" "), None),
    (EventType::Unparsed, Some(" `>"), None)])]
#[case("<` <` `>", vec![
    (EventType::VerbatimEscaping, Some(" <` "), None)])]
#[case("Foo<`Bar", vec![
    (EventType::Unparsed, Some("Foo"), None),
    (EventType::VerbatimEscaping, Some("Bar"), Some(HashSet::from(["F"])))])]
#[timeout(time::Duration::from_secs(1))]
fn it_works(#[case] input: &str, #[case] expected: Vec<EventCase>) {
    let parser = Parser::new(input.as_bytes(), 0);
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
