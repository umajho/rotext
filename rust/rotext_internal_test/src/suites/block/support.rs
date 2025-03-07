use std::panic::{catch_unwind, RefUnwindSafe};

use rotext_core::{Error, Event, EventType};

use crate::support::{FailedCase, FailureReason};

macro_rules! case {
    ($input_variants:expr, $expected:expr) => {
        case!(@__inner, $input_variants, $expected, false, false)
    };
    (@todo, $input_variants:expr, $expected:expr) => {
        case!(@__inner, $input_variants, $expected, true, false)
    };
    (@only, $input_variants:expr, $expected:expr) => {
        case!(@__inner, $input_variants, $expected, false, true)
    };
    (@__inner, $input_variants:expr, $expected:expr, $flag_todo:literal, $flag_only:literal) => {
        $crate::suites::block::support::Case {
            input_variants: $input_variants,
            expected: $expected,
            flags: $crate::suites::block::support::Flags {
                to_do: $flag_todo,
                only: $flag_only,
            },
        }
    };
}

pub(super) use case;

use super::Context;

pub(super) struct InternalContext<'a, TContext: Context> {
    external: &'a TContext,
    is_in_only_mode: bool,
}

type EventCase<'a> = (EventType, Option<&'a str>);

pub(super) struct GroupedCases {
    pub group: &'static str,
    pub cases: Vec<Case>,
}
impl GroupedCases {
    pub(super) fn collect_failed<TContext: Context + RefUnwindSafe>(
        &self,
        ctx: &TContext,
        is_in_only_mode: bool,
    ) -> Vec<FailedCase> {
        let ctx = InternalContext {
            external: ctx,
            is_in_only_mode,
        };

        self.cases
            .iter()
            .enumerate()
            .flat_map(|(i, case)| -> Vec<FailedCase> {
                case.collect_failed(&ctx, self.group, i + 1)
            })
            .collect()
    }

    pub(super) fn any_has_only_flag(&self) -> bool {
        self.cases.iter().any(|c| c.flags.only)
    }
}

pub(super) struct Case {
    pub input_variants: Vec<&'static str>,
    pub expected: Vec<EventMatcher>,
    pub flags: Flags,
}
pub(super) struct Flags {
    pub to_do: bool,
    pub only: bool,
}
impl Case {
    fn collect_failed<TContext: Context + RefUnwindSafe>(
        &self,
        ctx: &InternalContext<TContext>,
        group: &'static str,
        nth_case_in_group: usize,
    ) -> Vec<FailedCase> {
        self.input_variants
            .iter()
            .enumerate()
            .flat_map(|(i, input)| -> Vec<FailedCase> {
                let input = input.replace('␠', " ");
                self.collect_failed_auto_variant(ctx, group, nth_case_in_group, i + 1, input)
            })
            .collect()
    }

    fn collect_failed_auto_variant<TContext: Context + RefUnwindSafe>(
        &self,
        ctx: &InternalContext<TContext>,
        group: &'static str,
        nth_case_in_group: usize,
        nth_case_variant_in_case: usize,
        input: String,
    ) -> Vec<FailedCase> {
        AutoVariant::all()
            .iter()
            .filter_map(|auto_variant| -> Option<FailedCase> {
                if self.flags.to_do || (ctx.is_in_only_mode && !self.flags.only) {
                    let reason = if self.flags.to_do {
                        FailureReason::ToDo
                    } else {
                        FailureReason::Skipped
                    };
                    return Some(self.make_failed_case(
                        group,
                        nth_case_in_group,
                        nth_case_variant_in_case,
                        auto_variant,
                        input.clone(),
                        reason,
                    ));
                }

                let panic = {
                    let input = input.clone();
                    catch_unwind(|| {
                        assert_auto_variant_ok(ctx, auto_variant.clone(), input, &self.expected)
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

fn assert_auto_variant_ok<TContext: Context>(
    ctx: &InternalContext<TContext>,
    variant: AutoVariant,
    input: String,
    expected: &Vec<EventMatcher>,
) {
    let input = match variant {
        AutoVariant::Normal => input.to_string(),
        AutoVariant::WithLeadingLineFeed => format!("\n{}", input),
        AutoVariant::WithTrailingLineFeed => format!("{}\n", input),
    };

    assert_parse_ok_and_output_matches(ctx.external, &input, expected)
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

pub fn assert_parse_ok_and_output_matches<TContext: Context>(
    ctx: &TContext,
    input: &str,
    expected: &Vec<EventMatcher>,
) {
    assert_parse_ok_and_output_matches_with_stack(ctx, input, expected);
}

pub fn assert_parse_ok_and_output_matches_with_stack<TContext: Context>(
    _ctx: &TContext,
    input: &str,
    expected: &Vec<EventMatcher>,
) {
    let actual: Vec<_> = TContext::parse(input)
        .map(|ev| -> EventCase {
            let ev: Event = ev.unwrap();
            (
                EventType::from(ev.discriminant()),
                ev.content(input.as_bytes()),
            )
        })
        .collect();

    assert_eq!(expected, &actual)
}

pub fn assert_parse_error_with_stack<TContext: Context>(
    _ctx: &TContext,
    input: &str,
    expected_error: Error,
) {
    let actual: Result<Vec<_>, _> = TContext::parse(input).collect();

    assert_eq!(expected_error, actual.unwrap_err())
}
