#![cfg(test)]

mod groups_block_quote;
mod groups_code_block;
mod groups_description_list;
mod groups_heading;
mod groups_horizontal_rule;
mod groups_list;
mod groups_paragraph;
mod utils;

use std::panic::catch_unwind;

use utils::case;

use super::*;

use crate::{
    events::{Event, EventType},
    test_utils::{report_failed_cases, FaildCase},
};

type EventCase<'a> = (EventType, Option<&'a str>);

#[test]
fn it_works() {
    let table = {
        let mut table: Vec<GroupedCases> = vec![];

        table.push(GroupedCases {
            group: "空",
            cases: vec![case!(vec![""], vec![])],
        });
        table.extend(groups_paragraph::groups_paragraph());
        table.extend(groups_horizontal_rule::groups_horizontal_rule());
        table.extend(groups_heading::groups_heading());
        table.extend(groups_block_quote::groups_block_quote());
        table.extend(groups_list::groups_list());
        table.extend(groups_description_list::groups_description_list());
        table.extend(groups_code_block::groups_code_block());

        table
    };

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
            .flat_map(|(i, case)| -> Vec<FaildCase> { case.collect_failed(self.group, i + 1) })
            .collect()
    }
}

struct Case {
    input_variants: Vec<&'static str>,
    expected: Vec<EventMatcher>,
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

    let global_parser = global::Parser::new(input.as_bytes(), global::NewParserOptions::default());
    let block_parser = Parser::new(input.as_bytes(), global_parser);

    let actual: Vec<_> = block_parser
        .map(|ev| -> EventCase {
            let ev: Event = ev.into();
            (
                EventType::from(ev.discriminant()),
                ev.content(input.as_bytes()),
            )
        })
        .collect();

    assert_eq!(expected, &actual)
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
