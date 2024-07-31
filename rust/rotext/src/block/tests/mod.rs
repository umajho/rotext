#![cfg(test)]

mod groups_block_quote;
mod groups_code_block;
mod groups_description_list;
mod groups_heading;
mod groups_horizontal_rule;
mod groups_list;
mod groups_paragraph;
mod groups_table;
mod support;

use crate::{
    events::EventType,
    test_support::{report_failed_cases, FaildCase},
    utils::stack::ArrayStack,
    Error,
};
use support::{
    assert_parse_error_with_stack, assert_parse_ok_and_output_maches_with_stack, case, GroupedCases,
};

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
        table.extend(groups_table::groups_table());

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

#[test]
fn it_works_with_array_stack() {
    assert_parse_ok_and_output_maches_with_stack::<ArrayStack<_, 2>>("", &vec![]);
    assert_parse_ok_and_output_maches_with_stack::<ArrayStack<_, 2>>(
        ">",
        &vec![
            (EventType::EnterBlockQuote, None),
            (EventType::ExitBlock, None),
        ],
    );
    assert_parse_ok_and_output_maches_with_stack::<ArrayStack<_, 2>>(
        "> >",
        &vec![
            (EventType::EnterBlockQuote, None),
            (EventType::EnterBlockQuote, None),
            (EventType::ExitBlock, None),
            (EventType::ExitBlock, None),
        ],
    );
    assert_parse_ok_and_output_maches_with_stack::<ArrayStack<_, 2>>(
        "> > foo",
        &vec![
            (EventType::EnterBlockQuote, None),
            (EventType::EnterBlockQuote, None),
            (EventType::EnterParagraph, None),
            (EventType::Unparsed, Some("foo")),
            (EventType::ExitBlock, None),
            (EventType::ExitBlock, None),
            (EventType::ExitBlock, None),
        ],
    );
    assert_parse_error_with_stack::<ArrayStack<_, 2>>("> > >", Error::OutOfStackSpace)
}
