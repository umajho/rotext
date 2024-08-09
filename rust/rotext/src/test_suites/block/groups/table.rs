use indoc::indoc;

use crate::{
    events::EventType,
    test_suites::block::support::{case, GroupedCases},
};

pub fn groups_table() -> Vec<GroupedCases> {
    vec![
        GroupedCases {
            group: "表格>无内容",
            cases: vec![
                case!(
                    vec![
                        "{||}",
                        "{| |}",
                        indoc! {"
                        {|
                        |}"},
                    ],
                    vec![(EventType::EnterTable, None), (EventType::ExitBlock, None)]
                ),
                case!(
                    vec![
                        "{||-|}",
                        "{| |- |}",
                        indoc! {"
                        {|
                        |-
                        |}"},
                    ],
                    vec![
                        (EventType::EnterTable, None),
                        (EventType::IndicateTableRow, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![
                        "{||||}",
                        "{| || |}",
                        indoc! {"
                        {|
                        ||
                        |}"},
                    ],
                    vec![
                        (EventType::EnterTable, None),
                        (EventType::IndicateTableDataCell, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![
                        "{||-|||}",
                        "{| |- || |}",
                        indoc! {"
                        {|
                        |-||
                        |}"},
                        indoc! {"
                        {|
                        |-
                        ||
                        |}"},
                    ],
                    vec![
                        (EventType::EnterTable, None),
                        (EventType::IndicateTableRow, None),
                        (EventType::IndicateTableDataCell, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
            ],
        },
        GroupedCases {
            group: "表格",
            cases: vec![
                case!(
                    vec![
                        "{||-!!FOO!!BAR|-||baz||qux|}",
                        indoc! {"
                        {|
                        |-
                        !! FOO
                        !! BAR
                        |-
                        || baz
                        || qux
                        |}"},
                        indoc! {"
                        {|
                        |-
                        !!
                        FOO
                        !!
                        BAR
                        |-
                        ||
                        baz
                        ||
                        qux
                        |}"},
                        indoc! {"
                        {|
                        |-
                        !! FOO !! BAR
                        |-
                        || baz || qux
                        |}"}
                    ],
                    vec![
                        (EventType::EnterTable, None),
                        (EventType::IndicateTableRow, None),
                        (EventType::IndicateTableHeaderCell, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("FOO")),
                        (EventType::ExitBlock, None),
                        (EventType::IndicateTableHeaderCell, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("BAR")),
                        (EventType::ExitBlock, None),
                        (EventType::IndicateTableRow, None),
                        (EventType::IndicateTableDataCell, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("baz")),
                        (EventType::ExitBlock, None),
                        (EventType::IndicateTableDataCell, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("qux")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![
                        "{|!!FOO!!BAR|-baz||qux|}",
                        indoc! {"
                        {|
                        !! FOO
                        !! BAR
                        |-
                        baz
                        || qux
                        |}"},
                        indoc! {"
                        {|
                        !!
                        FOO
                        !!
                        BAR
                        |-
                        baz
                        ||
                        qux
                        |}"},
                        indoc! {"
                        {|
                        !! FOO !! BAR
                        |-
                        baz || qux
                        |}"},
                    ],
                    vec![
                        (EventType::EnterTable, None),
                        (EventType::IndicateTableHeaderCell, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("FOO")),
                        (EventType::ExitBlock, None),
                        (EventType::IndicateTableHeaderCell, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("BAR")),
                        (EventType::ExitBlock, None),
                        (EventType::IndicateTableRow, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("baz")),
                        (EventType::ExitBlock, None),
                        (EventType::IndicateTableDataCell, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("qux")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![indoc! {"
                        {|
                        || line 1
                        line 2
                        |}"},],
                    vec![
                        (EventType::EnterTable, None),
                        (EventType::IndicateTableDataCell, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("line 1")),
                        (EventType::NewLine, None),
                        (EventType::Unparsed, Some("line 2")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
            ],
        },
        GroupedCases {
            group: "表格>标题",
            cases: vec![
                case!(
                    vec![
                        indoc! {"
                        {|
                        |+ CAPTION
                        |}"},
                        "{| |+ CAPTION |}",
                        "{||+CAPTION|}",
                    ],
                    vec![
                        (EventType::EnterTable, None),
                        (EventType::IndicateTableCaption, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("CAPTION")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![indoc! {"
                        {|
                        |+ CAPTION
                        |-
                        |}"},],
                    vec![
                        (EventType::EnterTable, None),
                        (EventType::IndicateTableCaption, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("CAPTION")),
                        (EventType::ExitBlock, None),
                        (EventType::IndicateTableRow, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![indoc! {"
                        {|
                        |+ CAPTION
                        ||
                        |}"},],
                    vec![
                        (EventType::EnterTable, None),
                        (EventType::IndicateTableCaption, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("CAPTION")),
                        (EventType::ExitBlock, None),
                        (EventType::IndicateTableDataCell, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![indoc! {"
                        {|
                        ||
                        |+ NOT CAPTION
                        |}"},],
                    vec![
                        (EventType::EnterTable, None),
                        (EventType::IndicateTableDataCell, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("|+ NOT CAPTION")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
            ],
        },
        GroupedCases {
            group: "表格>item-likes 中的表格",
            cases: vec![
                case!(
                    vec![
                        indoc! {"
                        > {|
                        > !! FOO
                        > |}"},
                        indoc! {"
                        > {|
                        >
                        > !! FOO
                        >
                        > |}"},
                    ],
                    vec![
                        (EventType::EnterBlockQuote, None),
                        (EventType::EnterTable, None),
                        (EventType::IndicateTableHeaderCell, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("FOO")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![indoc! {"
                        * {|
                        > !! FOO
                        > |}"},],
                    vec![
                        (EventType::EnterUnorderedList, None),
                        (EventType::EnterListItem, None),
                        (EventType::EnterTable, None),
                        (EventType::IndicateTableHeaderCell, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("FOO")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![indoc! {"
                        > {|
                        > !! FOO
                        |}"},],
                    vec![
                        (EventType::EnterBlockQuote, None),
                        (EventType::EnterTable, None),
                        (EventType::IndicateTableHeaderCell, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("FOO")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("|}")),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![indoc! {"
                        * {|
                        > !! FOO
                        * |}"},],
                    vec![
                        (EventType::EnterUnorderedList, None),
                        (EventType::EnterListItem, None),
                        (EventType::EnterTable, None),
                        (EventType::IndicateTableHeaderCell, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("FOO")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::EnterListItem, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("|}")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
            ],
        },
        GroupedCases {
            group: "表格>表格中的先前测试过的 item-likes 以外的块级元素",
            cases: vec![
                case!(
                    vec!["{|---|}", "{| --- |}",],
                    vec![
                        (EventType::EnterTable, None),
                        (EventType::ThematicBreak, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![indoc! {"
                        {|
                        || --- ||
                        ---
                        |}"},],
                    vec![
                        (EventType::EnterTable, None),
                        (EventType::IndicateTableDataCell, None),
                        (EventType::ThematicBreak, None),
                        (EventType::IndicateTableDataCell, None),
                        (EventType::ThematicBreak, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    @todo,
                    vec!["{|== foo ==|}", "{| == foo == |}",],
                    vec![
                        (EventType::EnterTable, None),
                        (EventType::EnterHeading2, None),
                        (EventType::Unparsed, Some("foo")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![
                        indoc! {"
                        {|
                        ||
                        == foo
                        || bar ==
                        |}"},
                        indoc! {"
                        {|
                        || == foo || bar ==
                        |}"},
                    ],
                    vec![
                        (EventType::EnterTable, None),
                        (EventType::IndicateTableDataCell, None),
                        (EventType::EnterHeading2, None),
                        (EventType::Unparsed, Some("foo")),
                        (EventType::ExitBlock, None),
                        (EventType::IndicateTableDataCell, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("bar ==")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![
                        indoc! {"
                        {|
                        ||
                        ```info
                        code
                        ```
                        |}"},
                        indoc! {"
                        {|
                        || ```info
                        code
                        ```|}"},
                    ],
                    vec![
                        (EventType::EnterTable, None),
                        (EventType::IndicateTableDataCell, None),
                        (EventType::EnterCodeBlock, None),
                        (EventType::Text, Some("info")),
                        (EventType::IndicateCodeBlockCode, None),
                        (EventType::Text, Some("code")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
            ],
        },
        GroupedCases {
            group: "表格>相互嵌套的表格与 item-likes",
            cases: vec![
                case!(
                    vec![indoc! {"
                        > {|
                        > !! FOO
                        >
                        > > {|
                        > > !! BAR
                        > > |}
                        > |}"},],
                    vec![
                        (EventType::EnterBlockQuote, None),
                        (EventType::EnterTable, None),
                        (EventType::IndicateTableHeaderCell, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("FOO")),
                        (EventType::ExitBlock, None),
                        (EventType::EnterBlockQuote, None),
                        (EventType::EnterTable, None),
                        (EventType::IndicateTableHeaderCell, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("BAR")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![indoc! {"
                        {| > foo
                        || > bar |}"
                    },],
                    vec![
                        (EventType::EnterTable, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("> foo")),
                        (EventType::ExitBlock, None),
                        (EventType::IndicateTableDataCell, None),
                        (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("> bar")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
            ],
        },
    ]
}
