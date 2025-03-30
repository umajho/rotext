use indoc::indoc;

use rotext_core::EventType;

use crate::suites::block::support::{case, GroupedCases};

pub fn groups_code_block() -> Vec<GroupedCases> {
    vec![
        GroupedCases {
            group: "代码块",
            cases: vec![
                case!(
                    vec![
                        indoc! {"
                        ```
                        code
                        ```"},
                        indoc! {"
                        ```
                        code
                        ````"},
                        indoc! {"
                        ````
                        code
                        ````"},
                    ],
                    vec![
                        (EventType::EnterCodeBlock, None),
                        (EventType::IndicateCodeBlockCode, None),
                        (EventType::Text, Some("code")),
                        (EventType::NewLine, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![indoc! {"
                        ``
                        code
                        ```"},],
                    vec![
                        (EventType::EnterParagraph, None),
                        (EventType::__Unparsed, Some("``")),
                        (EventType::NewLine, None),
                        (EventType::__Unparsed, Some("code")),
                        (EventType::NewLine, None),
                        (EventType::__Unparsed, Some("```")),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![
                        indoc! {"
                        ```
                        ␣␣code␣␣
                        ```"},
                        indoc! {"
                        ```
                        ␣␣code␣␣
                        ````"},
                        indoc! {"
                        ````
                        ␣␣code␣␣
                        ````"},
                    ],
                    vec![
                        (EventType::EnterCodeBlock, None),
                        (EventType::IndicateCodeBlockCode, None),
                        (EventType::Text, Some("  code  ")),
                        (EventType::NewLine, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![
                        indoc! {"
                        ```
                        ```"},
                        indoc! {"
                        ```
                        ````"},
                        indoc! {"
                        ````
                        ````"},
                        "```",
                        indoc! {"
                        ````
                        "},
                    ],
                    vec![
                        (EventType::EnterCodeBlock, None),
                        (EventType::IndicateCodeBlockCode, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![indoc! {"
                        ```

                        ```"},],
                    vec![
                        (EventType::EnterCodeBlock, None),
                        (EventType::IndicateCodeBlockCode, None),
                        (EventType::NewLine, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![indoc! {"
                        ````
                        ```
                        ````"},],
                    vec![
                        (EventType::EnterCodeBlock, None),
                        (EventType::IndicateCodeBlockCode, None),
                        (EventType::Text, Some("```")),
                        (EventType::NewLine, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![indoc! {"
                        ```info
                        code
                        ```"},],
                    vec![
                        (EventType::EnterCodeBlock, None),
                        (EventType::Text, Some("info")),
                        (EventType::IndicateCodeBlockCode, None),
                        (EventType::Text, Some("code")),
                        (EventType::NewLine, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![indoc! {"
                        ```
                        code

                        ```"},],
                    vec![
                        (EventType::EnterCodeBlock, None),
                        (EventType::IndicateCodeBlockCode, None),
                        (EventType::Text, Some("code")),
                        (EventType::NewLine, None),
                        (EventType::NewLine, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![indoc! {"
                        ```
                        code


                        ```"},],
                    vec![
                        (EventType::EnterCodeBlock, None),
                        (EventType::IndicateCodeBlockCode, None),
                        (EventType::Text, Some("code")),
                        (EventType::NewLine, None),
                        (EventType::NewLine, None),
                        (EventType::NewLine, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![indoc! {"
                        ```
                        code
                        line␣2
                        ```"},],
                    vec![
                        (EventType::EnterCodeBlock, None),
                        (EventType::IndicateCodeBlockCode, None),
                        (EventType::Text, Some("code")),
                        (EventType::NewLine, None),
                        (EventType::Text, Some("line 2")),
                        (EventType::NewLine, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![indoc! {"
                        ```
                        code

                        line␣3
                        ```"},],
                    vec![
                        (EventType::EnterCodeBlock, None),
                        (EventType::IndicateCodeBlockCode, None),
                        (EventType::Text, Some("code")),
                        (EventType::NewLine, None),
                        (EventType::NewLine, None),
                        (EventType::Text, Some("line 3")),
                        (EventType::NewLine, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![indoc! {"
                        ```
                        code
                        ␣␣␣␣
                        line␣3
                        ```"},],
                    vec![
                        (EventType::EnterCodeBlock, None),
                        (EventType::IndicateCodeBlockCode, None),
                        (EventType::Text, Some("code")),
                        (EventType::NewLine, None),
                        (EventType::Text, Some("    ")),
                        (EventType::NewLine, None),
                        (EventType::Text, Some("line 3")),
                        (EventType::NewLine, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
            ],
        },
        GroupedCases {
            group: "代码块>代码块与全局阶段语法的互动>逐字转义",
            cases: vec![
                case!(
                    vec![indoc! {"
                        ```
                        <`␣```␣`>
                        ```"},],
                    vec![
                        (EventType::EnterCodeBlock, None),
                        (EventType::IndicateCodeBlockCode, None),
                        (EventType::VerbatimEscaping, Some("```")),
                        (EventType::NewLine, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![indoc! {"
                        ```info<`
                        info␣line␣2`>
                        ```"},],
                    vec![
                        (EventType::EnterCodeBlock, None),
                        (EventType::Text, Some("info")),
                        (EventType::VerbatimEscaping, Some("\ninfo line 2")),
                        (EventType::IndicateCodeBlockCode, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![indoc! {"
                        ```
                        ␣␣foo<`bar`>␣␣␣␣baz
                        ```"},],
                    vec![
                        (EventType::EnterCodeBlock, None),
                        (EventType::IndicateCodeBlockCode, None),
                        (EventType::Text, Some("  foo")),
                        (EventType::VerbatimEscaping, Some("bar")),
                        (EventType::Text, Some("    baz")),
                        (EventType::NewLine, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
            ],
        },
        GroupedCases {
            group: "代码块>代码块与全局阶段语法的互动>注释",
            cases: vec![
                case!(
                    vec![indoc! {"
                        ```
                        <%␣```␣%>
                        ```"},],
                    vec![
                        (EventType::EnterCodeBlock, None),
                        (EventType::IndicateCodeBlockCode, None),
                        (EventType::NewLine, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![indoc! {"
                        ```info<%
                        info␣line␣2%>
                        ```"},],
                    vec![
                        (EventType::EnterCodeBlock, None),
                        (EventType::Text, Some("info")),
                        (EventType::IndicateCodeBlockCode, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![indoc! {"
                        ```
                        ␣␣foo<%bar%>␣␣␣␣baz
                        ```"},],
                    vec![
                        (EventType::EnterCodeBlock, None),
                        (EventType::IndicateCodeBlockCode, None),
                        (EventType::Text, Some("  foo")),
                        (EventType::Text, Some("    baz")),
                        (EventType::NewLine, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
            ],
        },
        GroupedCases {
            group: "代码块>代码块于 list-like 中",
            cases: vec![
                case!(
                    vec![indoc! {"
                        >␠```info
                        >␠code
                        >␠```"},],
                    vec![
                        (EventType::EnterBlockQuote, None),
                        (EventType::EnterCodeBlock, None),
                        (EventType::Text, Some("info")),
                        (EventType::IndicateCodeBlockCode, None),
                        (EventType::Text, Some("code")),
                        (EventType::NewLine, None),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![indoc! {"
                        >␠```info
                        >␠code"},],
                    vec![
                        (EventType::EnterBlockQuote, None),
                        (EventType::EnterCodeBlock, None),
                        (EventType::Text, Some("info")),
                        (EventType::IndicateCodeBlockCode, None),
                        (EventType::Text, Some("code")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![indoc! {"
                        >␠```info
                        >␠>␣code
                        >␠```"},],
                    vec![
                        (EventType::EnterBlockQuote, None),
                        (EventType::EnterCodeBlock, None),
                        (EventType::Text, Some("info")),
                        (EventType::IndicateCodeBlockCode, None),
                        (EventType::Text, Some("> code")),
                        (EventType::NewLine, None),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![indoc! {"
                        >␠```info
                        >␠>␣code"},],
                    vec![
                        (EventType::EnterBlockQuote, None),
                        (EventType::EnterCodeBlock, None),
                        (EventType::Text, Some("info")),
                        (EventType::IndicateCodeBlockCode, None),
                        (EventType::Text, Some("> code")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![indoc! {"
                        >␠```info
                        >␠␣code
                        >␠```"},],
                    vec![
                        (EventType::EnterBlockQuote, None),
                        (EventType::EnterCodeBlock, None),
                        (EventType::Text, Some("info")),
                        (EventType::IndicateCodeBlockCode, None),
                        (EventType::Text, Some(" code")),
                        (EventType::NewLine, None),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![indoc! {"
                        >␠```info
                        >␠␣code
                        ```"},],
                    vec![
                        (EventType::EnterBlockQuote, None),
                        (EventType::EnterCodeBlock, None),
                        (EventType::Text, Some("info")),
                        (EventType::IndicateCodeBlockCode, None),
                        (EventType::Text, Some(" code")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                        (EventType::EnterCodeBlock, None),
                        (EventType::IndicateCodeBlockCode, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![indoc! {"
                        >␠```info
                        >␠1
                        >␠2"},],
                    vec![
                        (EventType::EnterBlockQuote, None),
                        (EventType::EnterCodeBlock, None),
                        (EventType::Text, Some("info")),
                        (EventType::IndicateCodeBlockCode, None),
                        (EventType::Text, Some("1")),
                        (EventType::NewLine, None),
                        (EventType::Text, Some("2")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
            ],
        },
        GroupedCases {
            group: "代码块>缩进",
            cases: vec![
                case!(
                    vec![
                        indoc! {"
                        ␠␠```
                        ␠␠code
                        ␠␠```"},
                        indoc! {"
                        ␠␠```
                        ␠␠code
                        ```"},
                        indoc! {"
                        ␠␠␠␠```
                        ␠␠code
                        ␠␠␠␠```"},
                        indoc! {"
                        ␠␠␠␠```
                        ␠␠code
                        ```"},
                    ],
                    vec![
                        (EventType::EnterCodeBlock, None),
                        (EventType::IndicateCodeBlockCode, None),
                        (EventType::Text, Some("code")),
                        (EventType::NewLine, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![indoc! {"
                        ␠␠```
                        ␠␠␣␣␣␣foo<`bar`>␣␣␣␣␣␣baz
                        ␠␠```"},],
                    vec![
                        (EventType::EnterCodeBlock, None),
                        (EventType::IndicateCodeBlockCode, None),
                        (EventType::Text, Some("    foo")),
                        (EventType::VerbatimEscaping, Some("bar")),
                        (EventType::Text, Some("      baz")),
                        (EventType::NewLine, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![
                        indoc! {"
                        >␠␠␠```
                        >␠␠␠code
                        >␠␠␠```"},
                        indoc! {"
                        >␠␠␠```
                        >␠␠␠code
                        >␠```"},
                        indoc! {"
                        >␠␠␠␠```
                        >␠␠code
                        >␠␠␠␠```"},
                        indoc! {"
                        >␠␠␠␠```
                        >␠␠code
                        >␠```"},
                    ],
                    vec![
                        (EventType::EnterBlockQuote, None),
                        (EventType::EnterCodeBlock, None),
                        (EventType::IndicateCodeBlockCode, None),
                        (EventType::Text, Some("code")),
                        (EventType::NewLine, None),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
            ],
        },
    ]
}
