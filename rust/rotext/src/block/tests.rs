#![cfg(test)]

use std::time;

use super::*;
use rstest::rstest;

use indoc::indoc;

use crate::events::{Event, EventType};

type EventCase<'a> = (EventType, Option<&'a str>);

#[rstest]
// ## 空
#[case(vec![""], vec![])]
// ## 段落
#[case(vec!["a", " a"], vec![
    (EventType::EnterParagraph, None),
        (EventType::Unparsed, Some("a")),
    (EventType::Exit, None)])]
#[case(vec!["a "], vec![
    (EventType::EnterParagraph, None),
        (EventType::Unparsed, Some("a ")),
    (EventType::Exit, None)])]
#[case(vec![
    indoc!{"
        a
        b"},
    indoc!{"
        a
        ␠b"},
], vec![
    (EventType::EnterParagraph, None),
        (EventType::Unparsed, Some("a")),
        (EventType::LineBreak, None),
        (EventType::Unparsed, Some("b")),
    (EventType::Exit, None)])]
#[case(vec![
    indoc!{"
        a

        b"},
    indoc!{"
        a

        ␠b"},
], vec![
    (EventType::EnterParagraph, None),
        (EventType::Unparsed, Some("a")),
    (EventType::Exit, None),
    (EventType::EnterParagraph, None),
        (EventType::Unparsed, Some("b")),
    (EventType::Exit, None)])]
// ### 段落与全局阶段语法的互动
#[case(vec!["a<`c`>"], vec![
    (EventType::EnterParagraph, None),
        (EventType::Unparsed, Some("a")),
        (EventType::Text, Some("c")),
    (EventType::Exit, None)])]
#[case(vec!["<`c`>b"], vec![
    (EventType::EnterParagraph, None),
        (EventType::Text, Some("c")),
        (EventType::Unparsed, Some("b")),
    (EventType::Exit, None)])]
#[case(vec!["a<`c`>b"], vec![
    (EventType::EnterParagraph, None),
        (EventType::Unparsed, Some("a")),
        (EventType::Text, Some("c")),
        (EventType::Unparsed, Some("b")),
    (EventType::Exit, None)])]
#[case(vec![
    indoc!{"
        a
        <`c`>"},
    indoc!{"
        a
        ␠<`c`>"},
], vec![
    (EventType::EnterParagraph, None),
        (EventType::Unparsed, Some("a")),
        (EventType::LineBreak, None),
        (EventType::Text, Some("c")),
    (EventType::Exit, None)])]
// ### “继续段落” 的优先级 “高于开启其他块级语法” 的优先级
#[case(vec![
    indoc!{"
        a
        ---"},
], vec![ // 分割线
    (EventType::EnterParagraph, None),
        (EventType::Unparsed, Some("a")),
        (EventType::LineBreak, None),
        (EventType::Unparsed, Some("---")),
    (EventType::Exit, None)])]
#[case(vec![
    indoc!{"
        a
        > b"},
], vec![ // 块引用
    (EventType::EnterParagraph, None),
        (EventType::Unparsed, Some("a")),
        (EventType::LineBreak, None),
        (EventType::Unparsed, Some("> b")),
    (EventType::Exit, None)])]
// ## 分割线
#[case(vec!["---", "----"], vec![
    (EventType::ThematicBreak, None)])]
#[case(vec!["--"], vec![
    (EventType::EnterParagraph, None),
        (EventType::Unparsed, Some("--")),
    (EventType::Exit, None)])]
#[case(vec![
    indoc!{"
        ---
        ---"},
    "--- ---",
], vec![
    (EventType::ThematicBreak, None),
    (EventType::ThematicBreak, None)])]
#[case(vec![
    indoc!{"
        ---
        a"},
    "---a", "--- a",
], vec![
    (EventType::ThematicBreak, None),
    (EventType::EnterParagraph, None),
        (EventType::Unparsed, Some("a")),
    (EventType::Exit, None)])]
// ### 分割线与全局阶段语法的互动
#[case(vec![
    indoc!{"
        ---
        <`a`>"},
    "---<`a`>", "--- <`a`>",
], vec![
    (EventType::ThematicBreak, None),
    (EventType::EnterParagraph, None),
        (EventType::Text, Some("a")),
    (EventType::Exit, None)])]
// ## 标题
#[case(vec!["= a ="], vec![
    (EventType::EnterHeading1, None),
        (EventType::Unparsed, Some("a")),
    (EventType::Exit, None)])]
#[case(vec!["== a =="], vec![
    (EventType::EnterHeading2, None),
        (EventType::Unparsed, Some("a")),
    (EventType::Exit, None)])]
#[case(vec!["=== a ==="], vec![
    (EventType::EnterHeading3, None),
        (EventType::Unparsed, Some("a")),
    (EventType::Exit, None)])]
#[case(vec!["==== a ===="], vec![
    (EventType::EnterHeading4, None),
        (EventType::Unparsed, Some("a")),
    (EventType::Exit, None)])]
#[case(vec!["===== a ====="], vec![
    (EventType::EnterHeading5, None),
        (EventType::Unparsed, Some("a")),
    (EventType::Exit, None)])]
#[case(vec!["====== a ======"], vec![
    (EventType::EnterHeading6, None),
        (EventType::Unparsed, Some("a")),
    (EventType::Exit, None)])]
#[case(vec!["== a"], vec![
    (EventType::EnterHeading2, None),
        (EventType::Unparsed, Some("a")),
    (EventType::Exit, None)])]
#[case(vec!["== a ="], vec![
    (EventType::EnterHeading2, None),
        (EventType::Unparsed, Some("a =")),
    (EventType::Exit, None)])]
#[case(vec!["== a ==="], vec![
    (EventType::EnterHeading2, None),
        (EventType::Unparsed, Some("a ===")),
    (EventType::Exit, None)])]
#[case(vec!["==  a  =="], vec![
    (EventType::EnterHeading2, None),
        (EventType::Unparsed, Some(" a ")),
    (EventType::Exit, None)])]
#[case(vec![
    indoc!{"
        == a ==
        b"},
    indoc!{"
        == a ==

        b"},
], vec![
    (EventType::EnterHeading2, None),
        (EventType::Unparsed, Some("a")),
    (EventType::Exit, None),
    (EventType::EnterParagraph, None),
        (EventType::Unparsed, Some("b")),
    (EventType::Exit, None)])]
#[case(vec![
    indoc!{"
        == a ==
        === b ==="},
    indoc!{"
        == a ==

        === b ==="},
], vec![
    (EventType::EnterHeading2, None),
        (EventType::Unparsed, Some("a")),
    (EventType::Exit, None),
    (EventType::EnterHeading3, None),
        (EventType::Unparsed, Some("b")),
    (EventType::Exit, None)])]
#[case(vec!["==a =="], vec![
    (EventType::EnterParagraph, None),
        (EventType::Unparsed, Some("==a ==")),
    (EventType::Exit, None)])]
#[case(vec!["== a=="], vec![
    (EventType::EnterHeading2, None),
        (EventType::Unparsed, Some("a==")),
    (EventType::Exit, None)])]
#[case(vec!["== a == "], vec![
    (EventType::EnterHeading2, None),
        (EventType::Unparsed, Some("a == ")),
    (EventType::Exit, None)])]
#[case(vec!["== a ==b"], vec![
    (EventType::EnterHeading2, None),
        (EventType::Unparsed, Some("a ==b")),
    (EventType::Exit, None)])]
#[case(vec!["== a == b =="], vec![
    (EventType::EnterHeading2, None),
        (EventType::Unparsed, Some("a == b")),
    (EventType::Exit, None)])]
#[case(vec!["======= a ======="], vec![
    (EventType::EnterParagraph, None),
        (EventType::Unparsed, Some("======= a =======")),
    (EventType::Exit, None)])]
#[case(vec!["== <`c`> =="], vec![
    (EventType::EnterHeading2, None),
        (EventType::Text, Some("c")),
    (EventType::Exit, None)])]
#[case(vec!["== a<`c`>b =="], vec![
    (EventType::EnterHeading2, None),
        (EventType::Unparsed, Some("a")),
        (EventType::Text, Some("c")),
        (EventType::Unparsed, Some("b")),
    (EventType::Exit, None)])]
// ## 块引用与其延续
#[case(vec!["> foo", " > foo", ">  foo"], vec![
    (EventType::EnterBlockQuote, None),
        (EventType::EnterParagraph, None),
            (EventType::Unparsed, Some("foo")),
        (EventType::Exit, None),
    (EventType::Exit, None)])]
#[case(vec![
    indoc!{"
        > foo
        > bar"},
    indoc!{"
        > foo
        >  bar"},
], vec![
    (EventType::EnterBlockQuote, None),
        (EventType::EnterParagraph, None),
            (EventType::Unparsed, Some("foo")),
            (EventType::LineBreak, None),
            (EventType::Unparsed, Some("bar")),
        (EventType::Exit, None),
    (EventType::Exit, None)])]
#[case(vec![
    indoc!{"
        > foo
        bar"},
    indoc!{"
        > foo
         bar"},
], vec![
    (EventType::EnterBlockQuote, None),
        (EventType::EnterParagraph, None),
            (EventType::Unparsed, Some("foo")),
        (EventType::Exit, None),
    (EventType::Exit, None),
    (EventType::EnterParagraph, None),
        (EventType::Unparsed, Some("bar")),
    (EventType::Exit, None)])]
#[case(vec![
    indoc!{"
        > foo

        > bar"},
    indoc!{"
        > foo

        >  bar"},
], vec![
    (EventType::EnterBlockQuote, None),
        (EventType::EnterParagraph, None),
            (EventType::Unparsed, Some("foo")),
        (EventType::Exit, None),
    (EventType::Exit, None),
    (EventType::EnterBlockQuote, None),
        (EventType::EnterParagraph, None),
            (EventType::Unparsed, Some("bar")),
        (EventType::Exit, None),
    (EventType::Exit, None)])]
#[case(vec!["> > foo", " > > foo"], vec![
    (EventType::EnterBlockQuote, None),
        (EventType::EnterBlockQuote, None),
            (EventType::EnterParagraph, None),
                (EventType::Unparsed, Some("foo")),
            (EventType::Exit, None),
        (EventType::Exit, None),
    (EventType::Exit, None)])]
#[case(vec![
    indoc!{"
        > > foo
        > bar"},
    indoc!{"
        > > foo
        >  bar"},
], vec![
    (EventType::EnterBlockQuote, None),
        (EventType::EnterBlockQuote, None),
            (EventType::EnterParagraph, None),
                (EventType::Unparsed, Some("foo")),
            (EventType::Exit, None),
        (EventType::Exit, None),
        (EventType::EnterParagraph, None),
            (EventType::Unparsed, Some("bar")),
        (EventType::Exit, None),
    (EventType::Exit, None)])]
#[case(vec![
    indoc!{"
        > foo
        > > bar"},
], vec![
    (EventType::EnterBlockQuote, None),
        (EventType::EnterParagraph, None),
            (EventType::Unparsed, Some("foo")),
            (EventType::LineBreak, None),
            (EventType::Unparsed, Some("> bar")),
        (EventType::Exit, None),
    (EventType::Exit, None)])]
// ### 块引用中的分割线与标题
#[case(vec!["> ---"], vec![
    (EventType::EnterBlockQuote, None),
        (EventType::ThematicBreak, None),
    (EventType::Exit, None)])]
#[case(vec![
    indoc!{"
        > ---
        ---"},
], vec![
    (EventType::EnterBlockQuote, None),
        (EventType::ThematicBreak, None),
    (EventType::Exit, None),
    (EventType::ThematicBreak, None)])]
#[case(vec!["> == a =="], vec![
    (EventType::EnterBlockQuote, None),
        (EventType::EnterHeading2, None),
            (EventType::Unparsed, Some("a")),
        (EventType::Exit, None),
    (EventType::Exit, None)])]
#[case(vec![
    indoc!{"
        > == a ==
        === b ==="},
], vec![
    (EventType::EnterBlockQuote, None),
        (EventType::EnterHeading2, None),
            (EventType::Unparsed, Some("a")),
        (EventType::Exit, None),
    (EventType::Exit, None),
    (EventType::EnterHeading3, None),
        (EventType::Unparsed, Some("b")),
    (EventType::Exit, None)])]
// ## 列表
#[case(vec!["# 1"], vec![
    (EventType::EnterOrderedList, None),
        (EventType::EnterListItem, None),
            (EventType::EnterParagraph, None),
                (EventType::Unparsed, Some("1")),
            (EventType::Exit, None),
        (EventType::Exit, None),
    (EventType::Exit, None)])]
#[case(vec!["* 1"], vec![
    (EventType::EnterUnorderedList, None),
        (EventType::EnterListItem, None),
            (EventType::EnterParagraph, None),
                (EventType::Unparsed, Some("1")),
            (EventType::Exit, None),
        (EventType::Exit, None),
    (EventType::Exit, None)])]
#[case(vec![
    indoc!{"
        # 1
        # 2"},
], vec![
    (EventType::EnterOrderedList, None),
        (EventType::EnterListItem, None),
            (EventType::EnterParagraph, None),
                (EventType::Unparsed, Some("1")),
            (EventType::Exit, None),
        (EventType::Exit, None),
        (EventType::EnterListItem, None),
            (EventType::EnterParagraph, None),
                (EventType::Unparsed, Some("2")),
            (EventType::Exit, None),
        (EventType::Exit, None),
    (EventType::Exit, None)])]
#[case(vec![
    indoc!{"
        * a
        * b
        * c"},
    ], vec![
    (EventType::EnterUnorderedList, None),
        (EventType::EnterListItem, None),
            (EventType::EnterParagraph, None),
                (EventType::Unparsed, Some("a")),
            (EventType::Exit, None),
        (EventType::Exit, None),
        (EventType::EnterListItem, None),
            (EventType::EnterParagraph, None),
                (EventType::Unparsed, Some("b")),
            (EventType::Exit, None),
        (EventType::Exit, None),
        (EventType::EnterListItem, None),
            (EventType::EnterParagraph, None),
                (EventType::Unparsed, Some("c")),
            (EventType::Exit, None),
        (EventType::Exit, None),
    (EventType::Exit, None)])]
#[case(vec![
    indoc!{"
        # 1

        # 2"},
    ], vec![
    (EventType::EnterOrderedList, None),
        (EventType::EnterListItem, None),
            (EventType::EnterParagraph, None),
                (EventType::Unparsed, Some("1")),
            (EventType::Exit, None),
        (EventType::Exit, None),
    (EventType::Exit, None),
    (EventType::EnterOrderedList, None),
        (EventType::EnterListItem, None),
            (EventType::EnterParagraph, None),
                (EventType::Unparsed, Some("2")),
            (EventType::Exit, None),
        (EventType::Exit, None),
    (EventType::Exit, None)])]
#[case(vec!["# # 1.1"], vec![
    (EventType::EnterOrderedList, None),
        (EventType::EnterListItem, None),
            (EventType::EnterOrderedList, None),
                (EventType::EnterListItem, None),
                    (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("1.1")),
                    (EventType::Exit, None),
                (EventType::Exit, None),
            (EventType::Exit, None),
        (EventType::Exit, None),
    (EventType::Exit, None)])]
#[case(vec![
    indoc!{"
        # # 1.1
        # 2"},
    ], vec![
    (EventType::EnterOrderedList, None),
        (EventType::EnterListItem, None),
            (EventType::EnterOrderedList, None),
                (EventType::EnterListItem, None),
                    (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("1.1")),
                    (EventType::Exit, None),
                (EventType::Exit, None),
            (EventType::Exit, None),
        (EventType::Exit, None),
        (EventType::EnterListItem, None),
            (EventType::EnterParagraph, None),
                (EventType::Unparsed, Some("2")),
            (EventType::Exit, None),
        (EventType::Exit, None),
    (EventType::Exit, None)])]
#[case(vec![
    indoc!{"
        # # 1.1
        #
        # 2"},
    ], vec![
    (EventType::EnterOrderedList, None),
        (EventType::EnterListItem, None),
            (EventType::EnterOrderedList, None),
                (EventType::EnterListItem, None),
                    (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("1.1")),
                    (EventType::Exit, None),
                (EventType::Exit, None),
            (EventType::Exit, None),
        (EventType::Exit, None),
        (EventType::EnterListItem, None),
        (EventType::Exit, None),
        (EventType::EnterListItem, None),
            (EventType::EnterParagraph, None),
                (EventType::Unparsed, Some("2")),
            (EventType::Exit, None),
        (EventType::Exit, None),
    (EventType::Exit, None)])]
#[case(vec![
    indoc!{"
        # 1
        # # 2.1"},
    ], vec![
    (EventType::EnterOrderedList, None),
        (EventType::EnterListItem, None),
            (EventType::EnterParagraph, None),
                (EventType::Unparsed, Some("1")),
            (EventType::Exit, None),
        (EventType::Exit, None),
        (EventType::EnterListItem, None),
            (EventType::EnterOrderedList, None),
                (EventType::EnterListItem, None),
                    (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("2.1")),
                    (EventType::Exit, None),
                (EventType::Exit, None),
            (EventType::Exit, None),
        (EventType::Exit, None),
    (EventType::Exit, None)])]
#[case(vec![
    indoc!{"
        # 1
        #
        # # 2.1"},
    ], vec![
    (EventType::EnterOrderedList, None),
        (EventType::EnterListItem, None),
            (EventType::EnterParagraph, None),
                (EventType::Unparsed, Some("1")),
            (EventType::Exit, None),
        (EventType::Exit, None),
        (EventType::EnterListItem, None),
        (EventType::Exit, None),
        (EventType::EnterListItem, None),
            (EventType::EnterOrderedList, None),
                (EventType::EnterListItem, None),
                    (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("2.1")),
                    (EventType::Exit, None),
                (EventType::Exit, None),
            (EventType::Exit, None),
        (EventType::Exit, None),
    (EventType::Exit, None)])]
#[case(vec![
    indoc!{"
        # 1
        # # 2.1
        # 3"},
    ], vec![
    (EventType::EnterOrderedList, None),
        (EventType::EnterListItem, None),
            (EventType::EnterParagraph, None),
                (EventType::Unparsed, Some("1")),
            (EventType::Exit, None),
        (EventType::Exit, None),
        (EventType::EnterListItem, None),
            (EventType::EnterOrderedList, None),
                (EventType::EnterListItem, None),
                    (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("2.1")),
                    (EventType::Exit, None),
                (EventType::Exit, None),
            (EventType::Exit, None),
        (EventType::Exit, None),
        (EventType::EnterListItem, None),
            (EventType::EnterParagraph, None),
                (EventType::Unparsed, Some("3")),
            (EventType::Exit, None),
        (EventType::Exit, None),
    (EventType::Exit, None)])]
#[case(vec![
    indoc!{"
        # # 1.1
        # 2
        # # 3.1"},
    ], vec![
    (EventType::EnterOrderedList, None),
        (EventType::EnterListItem, None),
            (EventType::EnterOrderedList, None),
                (EventType::EnterListItem, None),
                    (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("1.1")),
                    (EventType::Exit, None),
                (EventType::Exit, None),
            (EventType::Exit, None),
        (EventType::Exit, None),
        (EventType::EnterListItem, None),
            (EventType::EnterParagraph, None),
                (EventType::Unparsed, Some("2")),
            (EventType::Exit, None),
        (EventType::Exit, None),
        (EventType::EnterListItem, None),
            (EventType::EnterOrderedList, None),
                (EventType::EnterListItem, None),
                    (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("3.1")),
                    (EventType::Exit, None),
                (EventType::Exit, None),
            (EventType::Exit, None),
        (EventType::Exit, None),
    (EventType::Exit, None)])]
// ### 不同种类的列表
#[case(vec![
    indoc!{"
        # 1
        * a"},
    indoc!{"
        # 1

        * a"},
    ], vec![
    (EventType::EnterOrderedList, None),
        (EventType::EnterListItem, None),
            (EventType::EnterParagraph, None),
                (EventType::Unparsed, Some("1")),
            (EventType::Exit, None),
        (EventType::Exit, None),
    (EventType::Exit, None),
    (EventType::EnterUnorderedList, None),
        (EventType::EnterListItem, None),
            (EventType::EnterParagraph, None),
                (EventType::Unparsed, Some("a")),
            (EventType::Exit, None),
        (EventType::Exit, None),
    (EventType::Exit, None)])]
// ### 列表的延续
#[case(vec![
    indoc!{"
        # a
        > b"},
    ], vec![
    (EventType::EnterOrderedList, None),
        (EventType::EnterListItem, None),
            (EventType::EnterParagraph, None),
                (EventType::Unparsed, Some("a")),
                (EventType::LineBreak, None),
                (EventType::Unparsed, Some("b")),
            (EventType::Exit, None),
        (EventType::Exit, None),
    (EventType::Exit, None)])]
#[case(vec![
    indoc!{"
        # a
        >
        > b"},
    ], vec![
    (EventType::EnterOrderedList, None),
        (EventType::EnterListItem, None),
            (EventType::EnterParagraph, None),
                (EventType::Unparsed, Some("a")),
            (EventType::Exit, None),
            (EventType::EnterParagraph, None),
                (EventType::Unparsed, Some("b")),
            (EventType::Exit, None),
        (EventType::Exit, None),
    (EventType::Exit, None)])]
#[case(vec![
    indoc!{"
        # # 1.1
        > # 1.2"},
    ], vec![
    (EventType::EnterOrderedList, None),
        (EventType::EnterListItem, None),
            (EventType::EnterOrderedList, None),
                (EventType::EnterListItem, None),
                    (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("1.1")),
                    (EventType::Exit, None),
                (EventType::Exit, None),
                (EventType::EnterListItem, None),
                    (EventType::EnterParagraph, None),
                        (EventType::Unparsed, Some("1.2")),
                    (EventType::Exit, None),
                (EventType::Exit, None),
            (EventType::Exit, None),
        (EventType::Exit, None),
    (EventType::Exit, None)])]
// ## 代码块
#[case(vec![
    indoc!{"
        ```
        code
        ```"},
    indoc!{"
        ```
        code
        ````"},
    indoc!{"
        ````
        code
        ````"},
], vec![
    (EventType::EnterCodeBlock, None),
        (EventType::Separator, None),
        (EventType::Text, Some("code")),
    (EventType::Exit, None)])]
#[case(vec![
    indoc!{"
        ``
        code
        ```"},
], vec![
    (EventType::EnterParagraph, None),
        (EventType::Unparsed, Some("``")),
        (EventType::LineBreak, None),
        (EventType::Unparsed, Some("code")),
        (EventType::LineBreak, None),
        (EventType::Unparsed, Some("```")),
    (EventType::Exit, None)])]
#[case(vec![
    indoc!{"
        ```
        ␠␠code␠␠
        ```"},
    indoc!{"
        ```
        ␠␠code␠␠
        ````"},
    indoc!{"
        ````
        ␠␠code␠␠
        ````"},
], vec![
    (EventType::EnterCodeBlock, None),
        (EventType::Separator, None),
        (EventType::Text, Some("  code  ")),
    (EventType::Exit, None)])]
#[case(vec![
    indoc!{"
        ```
        ```"},
    indoc!{"
        ```
        ````"},
    indoc!{"
        ````
        ````"},
], vec![
    (EventType::EnterCodeBlock, None),
        (EventType::Separator, None),
    (EventType::Exit, None)])]
#[case(vec![
    indoc!{"
        ```

        ```"},
], vec![
    (EventType::EnterCodeBlock, None),
        (EventType::Separator, None),
        (EventType::LineBreak, None),
    (EventType::Exit, None)])]
#[case(vec![
    indoc!{"
        ````
        ```
        ````"},
], vec![
    (EventType::EnterCodeBlock, None),
        (EventType::Separator, None),
        (EventType::Text, Some("```")),
    (EventType::Exit, None)])]
#[case(vec![
    indoc!{"
        ```info
        code
        ```"},
], vec![
    (EventType::EnterCodeBlock, None),
        (EventType::Text, Some("info")),
        (EventType::Separator, None),
        (EventType::Text, Some("code")),
    (EventType::Exit, None)])]
#[case(vec![
    indoc!{"
        ```
        code

        ```"},
], vec![
    (EventType::EnterCodeBlock, None),
        (EventType::Separator, None),
        (EventType::Text, Some("code")),
        (EventType::LineBreak, None),
    (EventType::Exit, None)])]
#[case(vec![
    indoc!{"
        ```
        code


        ```"},
], vec![
    (EventType::EnterCodeBlock, None),
        (EventType::Separator, None),
        (EventType::Text, Some("code")),
        (EventType::LineBreak, None),
        (EventType::LineBreak, None),
    (EventType::Exit, None)])]
#[case(vec![
    indoc!{"
        ```
        code
        line 2
        ```"},
], vec![
    (EventType::EnterCodeBlock, None),
        (EventType::Separator, None),
        (EventType::Text, Some("code")),
        (EventType::LineBreak, None),
        (EventType::Text, Some("line 2")),
    (EventType::Exit, None)])]
#[case(vec![
    indoc!{"
        ```
        code

        line 3
        ```"},
], vec![
    (EventType::EnterCodeBlock, None),
        (EventType::Separator, None),
        (EventType::Text, Some("code")),
        (EventType::LineBreak, None),
        (EventType::LineBreak, None),
        (EventType::Text, Some("line 3")),
    (EventType::Exit, None)])]
#[case(vec![
    indoc!{"
        ```
        code
        ␠␠␠␠
        line 3
        ```"},
    ], vec![
    (EventType::EnterCodeBlock, None),
        (EventType::Separator, None),
        (EventType::Text, Some("code")),
        (EventType::LineBreak, None),
        (EventType::Text, Some("    ")),
        (EventType::LineBreak, None),
        (EventType::Text, Some("line 3")),
    (EventType::Exit, None)])]
// ### 代码块与全局阶段语法的互动
#[case(vec![
    indoc!{"
        ```
        <` ``` `>
        ```"},
], vec![
    (EventType::EnterCodeBlock, None),
        (EventType::Separator, None),
        (EventType::Text, Some("```")),
    (EventType::Exit, None)])]
#[case(vec![
    indoc!{"
        ```info<`
        info line 2`>
        ```"},
], vec![
    (EventType::EnterCodeBlock, None),
        (EventType::Text, Some("info")),
        (EventType::Text, Some("\ninfo line 2")),
        (EventType::Separator, None),
    (EventType::Exit, None)])]
// ### 代码块于 list-like 中
#[case(vec![
    indoc!{"
        > ```info
        > code
        > ```"},
    indoc!{"
        > ```info
        > code"},
], vec![
    (EventType::EnterBlockQuote, None),
        (EventType::EnterCodeBlock, None),
            (EventType::Text, Some("info")),
            (EventType::Separator, None),
            (EventType::Text, Some("code")),
        (EventType::Exit, None),
    (EventType::Exit, None)])]
#[case(vec![
    indoc!{"
        > ```info
        >  code
        > ```"},
], vec![
    (EventType::EnterBlockQuote, None),
        (EventType::EnterCodeBlock, None),
            (EventType::Text, Some("info")),
            (EventType::Separator, None),
            (EventType::Text, Some(" code")),
        (EventType::Exit, None),
    (EventType::Exit, None)])]
#[case(vec![
    indoc!{"
        > ```info
        >  code
        ```"},
], vec![
    (EventType::EnterBlockQuote, None),
        (EventType::EnterCodeBlock, None),
            (EventType::Text, Some("info")),
            (EventType::Separator, None),
            (EventType::Text, Some(" code")),
        (EventType::Exit, None),
    (EventType::Exit, None),
    (EventType::EnterCodeBlock, None),
        (EventType::Separator, None),
    (EventType::Exit, None)])]
#[timeout(time::Duration::from_secs(1))]
fn it_works(#[case] inputs: Vec<&str>, #[case] expected: Vec<EventCase>) {
    for (i, raw_input) in inputs.iter().enumerate() {
        let input = raw_input.replace('␠', " ");

        for variant in Variant::all() {
            let input = input.clone();

            let mut description = format!("sub case {}", i + 1);
            let input = match variant {
                Variant::Normal => {
                    description.push_str(&format!(":\n=begin\n{}\n=end", input));
                    input
                }
                Variant::WithLeadingLineFeed => {
                    description.push_str(".2 (with leading line feed)");
                    format!("\n{}", input)
                }
                Variant::WithTrailingLIneFeed => {
                    description.push_str(".2 (with trailing line feed)");
                    format!("{}\n", input)
                }
            };
            println!("{}", description);

            let global_parser = global::Parser::new(input.as_bytes(), 0);
            let block_parser = Parser::new(input.as_bytes(), global_parser);

            let actual: Vec<_> = block_parser
                .map(|ev| -> EventCase {
                    let ev: Event = ev.into();
                    (
                        EventType::from(ev.discriminant()),
                        ev.content(input.as_bytes()),
                    )
                })
                .collect();

            assert_eq!(expected, actual)
        }
    }
}

enum Variant {
    Normal,
    WithLeadingLineFeed,
    WithTrailingLIneFeed,
}
impl Variant {
    fn all() -> Vec<Variant> {
        vec![
            Variant::Normal,
            Variant::WithLeadingLineFeed,
            Variant::WithTrailingLIneFeed,
        ]
    }
}
