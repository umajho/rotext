use std::panic::catch_unwind;

use crate::{
    block::{Parser, StackEntry},
    events::EventType,
    global,
    test_support::{FailedCase, FailureReason},
    utils::stack::{Stack, VecStack},
    Error, Event,
};

macro_rules! case {
    ($input_variants:expr, $expected:expr) => {
        case!(@__inner, $input_variants, $expected, false)
    };
    (@todo, $input_variants:expr, $expected:expr) => {
            case!(@__inner, $input_variants, $expected, true)
        };
    (@__inner, $input_variants:expr, $expected:expr, $is_to_do:literal) => {
        $crate::block::tests::support::Case {
            input_variants: $input_variants,
            expected: $expected,
            is_to_do: $is_to_do,
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
    pub(super) fn collect_failed(&self) -> Vec<FailedCase> {
        self.cases
            .iter()
            .enumerate()
            .flat_map(|(i, case)| -> Vec<FailedCase> { case.collect_failed(self.group, i + 1) })
            .collect()
    }
}

pub(super) struct Case {
    pub input_variants: Vec<&'static str>,
    pub expected: Vec<EventMatcher>,
    pub is_to_do: bool,
}
impl Case {
    fn collect_failed(&self, group: &'static str, nth_case_in_group: usize) -> Vec<FailedCase> {
        self.input_variants
            .iter()
            .enumerate()
            .flat_map(|(i, input)| -> Vec<FailedCase> {
                let input = input.replace('␠', " ");
                self.collect_failed_auto_variant(group, nth_case_in_group, i + 1, input)
            })
            .collect()
    }

    fn collect_failed_auto_variant(
        &self,
        group: &'static str,
        nth_case_in_group: usize,
        nth_case_variant_in_case: usize,
        input: String,
    ) -> Vec<FailedCase> {
        AutoVariant::all()
            .iter()
            .filter_map(|auto_variant| -> Option<FailedCase> {
                if self.is_to_do {
                    return Some(self.make_failed_case(
                        group,
                        nth_case_in_group,
                        nth_case_variant_in_case,
                        auto_variant,
                        input.clone(),
                        FailureReason::ToDo,
                    ));
                }

                let panic = {
                    let input = input.clone();
                    catch_unwind(|| {
                        assert_auto_variant_ok(auto_variant.clone(), input, &self.expected)
                    })
                    .err()
                }?;

                Some(self.make_failed_case(
                    group,
                    nth_case_in_group,
                    nth_case_variant_in_case,
                    auto_variant,
                    input.clone(),
                    FailureReason::Panicked(panic),
                ))
            })
            .collect()
    }

    fn make_failed_case(
        &self,
        group: &'static str,
        nth_case_in_group: usize,
        nth_case_variant_in_case: usize,
        auto_variant: &AutoVariant,
        input: String,
        reason: FailureReason,
    ) -> FailedCase {
        FailedCase {
            group,
            nth_case_in_group,
            nth_case_variant_in_case: Some(nth_case_variant_in_case),
            auto_variant: Some(auto_variant.to_str()),
            input,
            reason,
        }
    }
}

type EventMatcher = (EventType, Option<&'static str>);

fn assert_auto_variant_ok(variant: AutoVariant, input: String, expected: &Vec<EventMatcher>) {
    let input = match variant {
        AutoVariant::Normal => input.to_string(),
        AutoVariant::WithLeadingLineFeed => format!("\n{}", input),
        AutoVariant::WithTrailingLineFeed => format!("{}\n", input),
    };

    assert_parse_ok_and_output_maches(&input, expected)
}

#[derive(Clone)]
enum AutoVariant {
    Normal,
    WithLeadingLineFeed,
    WithTrailingLineFeed,
}
impl AutoVariant {
    fn all() -> Vec<AutoVariant> {
        vec![
            AutoVariant::Normal,
            AutoVariant::WithLeadingLineFeed,
            AutoVariant::WithTrailingLineFeed,
        ]
    }

    fn to_str(&self) -> &'static str {
        match self {
            AutoVariant::Normal => "Normal",
            AutoVariant::WithLeadingLineFeed => "WithLeadingLineFeed",
            AutoVariant::WithTrailingLineFeed => "WithTrailingLIneFeed",
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
