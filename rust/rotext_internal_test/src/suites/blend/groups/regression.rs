use rotext_core::EventType;

use crate::suites::blend::support::{GroupedCases, case};

pub fn groups_regression() -> Vec<GroupedCases> {
    vec![GroupedCases {
        group: "回归",
        cases: vec![case!(vec!["{|␠!!␠foo_␠!!␠|}"], vec![
            (EventType::EnterTable, None),
            (EventType::IndicateTableHeaderCell, None),
            (EventType::EnterParagraph, None),
            (EventType::Text, Some("foo_")),
            (EventType::ExitBlock, None),
            (EventType::IndicateTableHeaderCell, None),
            (EventType::ExitBlock, None),
        ])],
    }]
}
