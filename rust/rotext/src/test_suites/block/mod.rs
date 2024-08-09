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

    let failed_cases: Vec<_> = table
        .iter()
        .flat_map(|row| -> Vec<FailedCase> { row.collect_failed(ctx) })
        .collect();

    let todos = failed_cases
        .iter()
        .filter(|c| matches!(c.reason, FailureReason::ToDo))
        .count();
    if todos > 0 {
        println!("({} TODO cases)", todos)
    }

    if failed_cases.len() == todos {
        return;
    }
    let faild_case_count = failed_cases.len() - todos;

    report_panicked_cases(failed_cases);

    panic!("{} cases failed!", faild_case_count);
}
