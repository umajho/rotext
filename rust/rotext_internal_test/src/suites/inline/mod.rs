mod groups;

use std::panic::RefUnwindSafe;

use rotext_core::Event;

use support::GroupedCases;

use crate::support::{report_panicked_cases, FailedCase, FailureReason};

pub mod support;

pub trait Context {
    /// 返回的事件应该都属于 `Inline` 分组。
    fn parse(input: &str) -> Vec<Event>;
}

pub fn run<TContext: Context + RefUnwindSafe>(ctx: &TContext) {
    let table = {
        let mut table: Vec<GroupedCases> = vec![];

        table.extend(groups::basic::groups_basic());
        table.extend(groups::ref_link::groups_ref_link());
        table.extend(groups::wiki_link::groups_wiki_link());
        table.extend(groups::dicexp::groups_dicexp());
        table.extend(groups::strong_and_strikethrough::groups_strong_and_strikethrough());
        table.extend(groups::ruby::groups_ruby());
        table.extend(groups::code_span::groups_code_span());

        table
    };

    let is_in_only_mode = table.iter().any(|g| g.any_has_only_flag());

    let failed_cases: Vec<_> = table
        .iter()
        .flat_map(|row| -> Vec<FailedCase> { row.collect_failed(ctx, is_in_only_mode) })
        .collect();

    let todos = failed_cases
        .iter()
        .filter(|c| matches!(c.reason, FailureReason::ToDo))
        .count();
    if todos > 0 {
        println!("({} TODO cases)", todos)
    }
    let skipped = failed_cases
        .iter()
        .filter(|c| matches!(c.reason, FailureReason::Skipped))
        .count();
    if skipped > 0 {
        println!("({} skipped cases)", skipped)
    }

    let actual_failed_case_count = failed_cases.len() - todos - skipped;
    if actual_failed_case_count == 0 {
        return;
    }

    report_panicked_cases(failed_cases);

    panic!("{} cases failed!", actual_failed_case_count);
}
