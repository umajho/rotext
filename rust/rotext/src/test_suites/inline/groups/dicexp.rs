use crate::{
    events::EventType,
    test_suites::inline::support::{case, GroupedCases},
};

pub fn groups_dicexp() -> Vec<GroupedCases> {
    vec![GroupedCases {
        group: "骰子表达式",
        cases: vec![
            case!(
                vec!["[=d100]", "[=d100"],
                vec![(EventType::Dicexp, Some("d100")),]
            ),
            case!(
                vec!["abc[=d100]"],
                vec![
                    (EventType::Text, Some("abc")),
                    (EventType::Dicexp, Some("d100")),
                ]
            ),
            case!(
                vec!["[=d100]abc"],
                vec![
                    (EventType::Dicexp, Some("d100")),
                    (EventType::Text, Some("abc")),
                ]
            ),
        ],
    }]
}
