#![cfg(test)]

use std::time;

use super::*;
use rstest::rstest;

use crate::events::{Event, EventType};

type EventCase<'a> = (EventType, Option<&'a str>);

#[rstest]
// ## 空
#[case(vec![""], vec![])]
// ## 段落
#[case(vec!["a", " a", "\na", "a\n"], vec![
    (EventType::EnterParagraph, None),
    (EventType::Unparsed, Some("a")),
    (EventType::Exit, None)])]
#[case(vec!["a "], vec![
    (EventType::EnterParagraph, None),
    (EventType::Unparsed, Some("a ")),
    (EventType::Exit, None)])]
#[case(vec!["a\nb", "a\n b"], vec![
    (EventType::EnterParagraph, None),
    (EventType::Unparsed, Some("a")),
    (EventType::LineBreak, None),
    (EventType::Unparsed, Some("b")),
    (EventType::Exit, None)])]
#[case(vec!["a\n\nb", "a\n\n b"], vec![
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
#[case(vec!["a\n<`c`>", "a\n <`c`>"], vec![
    (EventType::EnterParagraph, None),
    (EventType::Unparsed, Some("a")),
    (EventType::LineBreak, None),
    (EventType::Text, Some("c")),
    (EventType::Exit, None)])]
// ### “继续段落” 的优先级 “高于开启其他块级语法” 的优先级
#[case(vec!["a\n---"], vec![ // 分割线
    (EventType::EnterParagraph, None),
    (EventType::Unparsed, Some("a")),
    (EventType::LineBreak, None),
    (EventType::Unparsed, Some("---")),
    (EventType::Exit, None)])]
// ## 分割线
#[case(vec!["---", "----"], vec![
    (EventType::ThematicBreak, None)])]
#[case(vec!["--"], vec![
    (EventType::EnterParagraph, None),
    (EventType::Unparsed, Some("--")),
    (EventType::Exit, None)])]
#[case(vec!["---\n---", "--- ---"], vec![
    (EventType::ThematicBreak, None),
    (EventType::ThematicBreak, None)])]
#[case(vec!["---\na", "---a", "--- a"], vec![
    (EventType::ThematicBreak, None),
    (EventType::EnterParagraph, None),
    (EventType::Unparsed, Some("a")),
    (EventType::Exit, None)])]
// ### 分割线与全局阶段语法的互动
#[case(vec!["---\n<`a`>", "---<`a`>", "--- <`a`>"], vec![
    (EventType::ThematicBreak, None),
    (EventType::EnterParagraph, None),
    (EventType::Text, Some("a")),
    (EventType::Exit, None)])]
// ## 标题
#[case(vec!["= a ="], vec![
    (EventType::EnterHeading1, None),
    (EventType::Unparsed, Some("a")),
    (EventType::Exit, None)])]
#[case(vec!["== a ==", "\n== a ==", "== a ==\n", "== a ==\n\n"], vec![
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
#[case(vec!["== a ==\nb", "== a ==\n\nb"], vec![
    (EventType::EnterHeading2, None),
    (EventType::Unparsed, Some("a")),
    (EventType::Exit, None),
    (EventType::EnterParagraph, None),
    (EventType::Unparsed, Some("b")),
    (EventType::Exit, None)])]
#[case(vec!["== a ==\n=== b ===", "== a ==\n\n=== b ==="], vec![
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
// ## 代码块
#[case(vec!["```\ncode\n```", "```\ncode\n````", "````\ncode\n````"], vec![
    (EventType::EnterCodeBlock, None),
    (EventType::Separator, None),
    (EventType::Text, Some("code")),
    (EventType::Exit, None)])]
#[case(vec!["```\n  code  \n```", "```\n  code  \n````", "````\n  code  \n````"], vec![
    (EventType::EnterCodeBlock, None),
    (EventType::Separator, None),
    (EventType::Text, Some("  code  ")),
    (EventType::Exit, None)])]
#[case(vec!["```\n```", "```\n````", "````\n````"], vec![
    (EventType::EnterCodeBlock, None),
    (EventType::Separator, None),
    (EventType::Exit, None)])]
#[case(vec!["```\n\n```"], vec![
    (EventType::EnterCodeBlock, None),
    (EventType::Separator, None),
    (EventType::LineBreak, None),
    (EventType::Exit, None)])]
#[case(vec!["````\n```\n````"], vec![
    (EventType::EnterCodeBlock, None),
    (EventType::Separator, None),
    (EventType::Text, Some("```")),
    (EventType::Exit, None)])]
#[case(vec!["```info\ncode\n```"], vec![
    (EventType::EnterCodeBlock, None),
    (EventType::Text, Some("info")),
    (EventType::Separator, None),
    (EventType::Text, Some("code")),
    (EventType::Exit, None)])]
#[case(vec!["```\ncode\n\n```"], vec![
    (EventType::EnterCodeBlock, None),
    (EventType::Separator, None),
    (EventType::Text, Some("code")),
    (EventType::LineBreak, None),
    (EventType::Exit, None)])]
#[case(vec!["```\ncode\n\n\n```"], vec![
    (EventType::EnterCodeBlock, None),
    (EventType::Separator, None),
    (EventType::Text, Some("code")),
    (EventType::LineBreak, None),
    (EventType::LineBreak, None),
    (EventType::Exit, None)])]
#[case(vec!["```\ncode\nline 3\n```"], vec![
    (EventType::EnterCodeBlock, None),
    (EventType::Separator, None),
    (EventType::Text, Some("code")),
    (EventType::LineBreak, None),
    (EventType::Text, Some("line 3")),
    (EventType::Exit, None)])]
#[case(vec!["```\ncode\n\nline 3\n```"], vec![
    (EventType::EnterCodeBlock, None),
    (EventType::Separator, None),
    (EventType::Text, Some("code")),
    (EventType::LineBreak, None),
    (EventType::LineBreak, None),
    (EventType::Text, Some("line 3")),
    (EventType::Exit, None)])]
// ### 代码块与全局阶段语法的互动
#[case(vec!["```\n<` ``` `>\n```"], vec![
    (EventType::EnterCodeBlock, None),
    (EventType::Separator, None),
    (EventType::Text, Some("```")),
    (EventType::Exit, None)])]
#[case(vec!["```info<`\ninfo line 2`>\n```"], vec![
    (EventType::EnterCodeBlock, None),
    (EventType::Text, Some("info")),
    (EventType::Text, Some("\ninfo line 2")),
    (EventType::Separator, None),
    (EventType::Exit, None)])]
#[timeout(time::Duration::from_secs(1))]
fn it_works(#[case] inputs: Vec<&str>, #[case] expected: Vec<EventCase>) {
    for (i, input) in inputs.iter().enumerate() {
        println!("sub case {}:\n=begin\n{}\n=end", i + 1, input);

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
