use crate::{
    events::EventType,
    test_suites::inline::support::{case, GroupedCases},
};

pub fn groups_ref_link() -> Vec<GroupedCases> {
    vec![GroupedCases {
        group: "引用链接",
        cases: vec![
            case!(
                vec![">>TP.123"],
                vec![(EventType::RefLink, Some("TP.123")),]
            ),
            case!(
                vec!["abc>>TP.123"],
                vec![
                    (EventType::Text, Some("abc")),
                    (EventType::RefLink, Some("TP.123")),
                ]
            ),
            case!(
                vec![">>TP.123abc"],
                vec![
                    (EventType::RefLink, Some("TP.123")),
                    (EventType::Text, Some("abc")),
                ]
            ),
        ],
    }]
}
