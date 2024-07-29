use std::panic::catch_unwind;

use crate::{
    block::{Parser, StackEntry},
    events::EventType,
    global,
    test_utils::FaildCase,
    utils::stack::{Stack, VecStack},
    Error, Event,
};

macro_rules! case {
    ($input_variants:expr, $expected:expr) => {
        $crate::block::tests::utils::Case {
            input_variants: $input_variants,
            expected: $expected,
        }
    };
}

pub(super) use case;

type EventCase<'a> = (EventType, Option<&'a str>);

pub(super) struct GroupedCases {
    pub group: &'static str,
    pub cases: Vec<Case>,
}
impl GroupedCases {
    pub(super) fn collect_failed(&self) -> Vec<FaildCase> {
        self.cases
            .iter()
            .enumerate()
            .flat_map(|(i, case)| -> Vec<FaildCase> { case.collect_failed(self.group, i + 1) })
            .collect()
    }
}

pub(super) struct Case {
    pub input_variants: Vec<&'static str>,
    pub expected: Vec<EventMatcher>,
}
impl Case {
    fn collect_failed(&self, group: &'static str, nth_case_in_group: usize) -> Vec<FaildCase> {
        self.input_variants
            .iter()
            .enumerate()
            .flat_map(|(i, input)| -> Vec<FaildCase> {
                let input = input.replace('␠', " ");
                collect_failed_auto_variant(group, nth_case_in_group, i + 1, input, &self.expected)
            })
            .collect()
    }
}

type EventMatcher = (EventType, Option<&'static str>);

fn collect_failed_auto_variant(
    group: &'static str,
    nth_case_in_group: usize,
    nth_case_variant_in_case: usize,
    input: String,
    expected: &Vec<EventMatcher>,
) -> Vec<FaildCase> {
    AutoVariant::all()
        .iter()
        .filter_map(|variant| -> Option<FaildCase> {
            let panic = {
                let input = input.clone();
                catch_unwind(|| assert_auto_variant_ok(variant.clone(), input, expected)).err()
            }?;

            Some(FaildCase {
                group,
                nth_case_in_group,
                nth_case_variant_in_case: Some(nth_case_variant_in_case),
                auto_variant: Some(variant.to_str()),
                input: input.clone(),
                panic,
            })
        })
        .collect()
}

fn assert_auto_variant_ok(variant: AutoVariant, input: String, expected: &Vec<EventMatcher>) {
    let input = match variant {
        AutoVariant::Normal => input.to_string(),
        AutoVariant::WithLeadingLineFeed => format!("\n{}", input),
        AutoVariant::WithTrailingLIneFeed => format!("{}\n", input),
    };

    assert_parse_ok_and_output_maches(&input, expected)
}

#[derive(Clone)]
enum AutoVariant {
    Normal,
    WithLeadingLineFeed,
    WithTrailingLIneFeed,
}
impl AutoVariant {
    fn all() -> Vec<AutoVariant> {
        vec![
            AutoVariant::Normal,
            AutoVariant::WithLeadingLineFeed,
            AutoVariant::WithTrailingLIneFeed,
        ]
    }

    fn to_str(&self) -> &'static str {
        match self {
            AutoVariant::Normal => "Normal",
            AutoVariant::WithLeadingLineFeed => "WithLeadingLineFeed",
            AutoVariant::WithTrailingLIneFeed => "WithTrailingLIneFeed",
        }
    }
}

pub fn assert_parse_ok_and_output_maches(input: &str, expected: &Vec<EventMatcher>) {
    assert_parse_ok_and_output_maches_with_stack::<VecStack<_>>(input, expected);
}

pub fn assert_parse_ok_and_output_maches_with_stack<TStack: Stack<StackEntry>>(
    input: &str,
    expected: &Vec<EventMatcher>,
) {
    let global_parser = global::Parser::new(input.as_bytes(), global::NewParserOptions::default());
    let block_parser: Parser<TStack> = Parser::new(input.as_bytes(), global_parser);

    let actual: Vec<_> = block_parser
        .map(|ev| -> EventCase {
            let ev: Event = ev.unwrap().into();
            (
                EventType::from(ev.discriminant()),
                ev.content(input.as_bytes()),
            )
        })
        .collect();

    assert_eq!(expected, &actual)
}

pub fn assert_parse_error_with_stack<TStack: Stack<StackEntry>>(
    input: &str,
    expected_error: Error,
) {
    let global_parser = global::Parser::new(input.as_bytes(), global::NewParserOptions::default());
    let block_parser: Parser<TStack> = Parser::new(input.as_bytes(), global_parser);

    let actual: Result<Vec<_>, _> = block_parser.collect();

    assert_eq!(expected_error, actual.unwrap_err())
}
