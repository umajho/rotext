mod groups;
pub mod support;

use std::panic::RefUnwindSafe;

use support::{case, GroupedCases};

use crate::{
    test_support::{report_panicked_cases, FailedCase, FailureReason},
    BlockEvent,
};

pub trait Context {
    fn parse(input: &str) -> impl Iterator<Item = crate::Result<BlockEvent>>;
}

pub fn run<TContext: Context + RefUnwindSafe>(ctx: &TContext) {
    let table = {
        let mut table: Vec<GroupedCases> = vec![];

        table.push(GroupedCases {
            group: "空",
            cases: vec![case!(vec![""], vec![])],
        });
        table.extend(groups::paragraph::groups_paragraph());
        table.extend(groups::horizontal_rule::groups_horizontal_rule());
        table.extend(groups::heading::groups_heading());
        table.extend(groups::block_quote::groups_block_quote());
        table.extend(groups::list::groups_list());
        table.extend(groups::description_list::groups_description_list());
        table.extend(groups::code_block::groups_code_block());
        table.extend(groups::table::groups_table());

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
