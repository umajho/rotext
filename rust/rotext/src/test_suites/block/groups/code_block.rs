use indoc::indoc;

use crate::{
    events::EventType,
    test_suites::block::support::{case, GroupedCases},
};

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
                        (EventType::Unparsed, Some("``")),
                        (EventType::NewLine, None),
                        (EventType::Unparsed, Some("code")),
                        (EventType::NewLine, None),
                        (EventType::Unparsed, Some("```")),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![
                        indoc! {"
                        ```
                        ␠␠code␠␠
                        ```"},
                        indoc! {"
                        ```
                        ␠␠code␠␠
                        ````"},
                        indoc! {"
                        ````
                        ␠␠code␠␠
                        ````"},
                    ],
                    vec![
                        (EventType::EnterCodeBlock, None),
                        (EventType::IndicateCodeBlockCode, None),
                        (EventType::Text, Some("  code  ")),
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
                        line 2
                        ```"},],
                    vec![
                        (EventType::EnterCodeBlock, None),
                        (EventType::IndicateCodeBlockCode, None),
                        (EventType::Text, Some("code")),
                        (EventType::NewLine, None),
                        (EventType::Text, Some("line 2")),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![indoc! {"
                        ```
                        code

                        line 3
                        ```"},],
                    vec![
                        (EventType::EnterCodeBlock, None),
                        (EventType::IndicateCodeBlockCode, None),
                        (EventType::Text, Some("code")),
                        (EventType::NewLine, None),
                        (EventType::NewLine, None),
                        (EventType::Text, Some("line 3")),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![indoc! {"
                        ```
                        code
                        ␠␠␠␠
                        line 3
                        ```"},],
                    vec![
                        (EventType::EnterCodeBlock, None),
                        (EventType::IndicateCodeBlockCode, None),
                        (EventType::Text, Some("code")),
                        (EventType::NewLine, None),
                        (EventType::Text, Some("    ")),
                        (EventType::NewLine, None),
                        (EventType::Text, Some("line 3")),
                        (EventType::ExitBlock, None),
                    ]
                ),
            ],
        },
        GroupedCases {
            group: "代码块>代码块与全局阶段语法的互动",
            cases: vec![
                case!(
                    vec![indoc! {"
                        ```
                        <` ``` `>
                        ```"},],
                    vec![
                        (EventType::EnterCodeBlock, None),
                        (EventType::IndicateCodeBlockCode, None),
                        (EventType::VerbatimEscaping, Some("```")),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![indoc! {"
                        ```info<`
                        info line 2`>
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
                        <` ``` `>
                        ```"},],
                    vec![
                        (EventType::EnterCodeBlock, None),
                        (EventType::IndicateCodeBlockCode, None),
                        (EventType::VerbatimEscaping, Some("```")),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![indoc! {"
                        ```info<`
                        info line 2`>
                        ```"},],
                    vec![
                        (EventType::EnterCodeBlock, None),
                        (EventType::Text, Some("info")),
                        (EventType::VerbatimEscaping, Some("\ninfo line 2")),
                        (EventType::IndicateCodeBlockCode, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
            ],
        },
        GroupedCases {
            group: "代码块>代码块于 list-like 中",
            cases: vec![
                case!(
                    vec![
                        indoc! {"
                        > ```info
                        > code
                        > ```"},
                        indoc! {"
                        > ```info
                        > code"},
                    ],
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
                    vec![
                        indoc! {"
                        > ```info
                        > > code
                        > ```"},
                        indoc! {"
                        > ```info
                        > > code"},
                    ],
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
                        > ```info
                        >  code
                        > ```"},],
                    vec![
                        (EventType::EnterBlockQuote, None),
                        (EventType::EnterCodeBlock, None),
                        (EventType::Text, Some("info")),
                        (EventType::IndicateCodeBlockCode, None),
                        (EventType::Text, Some(" code")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![indoc! {"
                        > ```info
                        >  code
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
                        (EventType::ExitBlock, None),
                    ]
                ),
                case!(
                    vec![
                        indoc! {"
                        >   ```
                        >   code
                        >   ```"},
                        indoc! {"
                        >   ```
                        >   code
                        > ```"},
                        indoc! {"
                        >     ```
                        >   code
                        >     ```"},
                        indoc! {"
                        >     ```
                        >   code
                        > ```"},
                    ],
                    vec![
                        (EventType::EnterBlockQuote, None),
                        (EventType::EnterCodeBlock, None),
                        (EventType::IndicateCodeBlockCode, None),
                        (EventType::Text, Some("code")),
                        (EventType::ExitBlock, None),
                        (EventType::ExitBlock, None),
                    ]
                ),
            ],
        },
    ]
}
