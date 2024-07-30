#![cfg(test)]

#[allow(unused_imports)]
use crate::test_utils::{self, report_failed_cases, FaildCase, GroupedCases};

use super::*;

#[allow(unused_macros)]
macro_rules! case {
    ($input:expr, [$(($($ev:tt)*)),*,], $expected:expr,) => {
        Case {
            input_events: vec![$(case!(_input_event, $($ev)*)),*],
            input: $input,
            expected: $expected,

            options: CaseOptions {
                #[cfg(feature = "block-id")]
                with_block_id: false,
            },
        }
    };
    (_input_event, VerbatimEscaping ($start:literal..$end:literal)) => {
        BlendEvent::VerbatimEscaping($crate::events::VerbatimEscaping {
            content: case!(_range, $start..$end),
            is_closed_forcedly: false,
        })
    };
    (_input_event, NewLine (..)) => {
        BlendEvent::NewLine($crate::events::NewLine {})
    };
    (_input_event, Text ($start:literal..$end:literal)) => {
        BlendEvent::Text(case!(_range, $start..$end))
    };
    (_input_event, Separator ()) => {
        BlendEvent::Separator
    };
    (_input_event, ExitBlock (..)) => {
        BlendEvent::ExitBlock($crate::events::ExitBlock {})
    };
    (_input_event, ThematicBreak (..)) => {
        BlendEvent::ThematicBreak($crate::events::ThematicBreak {})
    };
    (_input_event, $v:tt (..)) => {
        BlendEvent::$v($crate::events::BlockWithID {})
    };
    (_input_event, $v:path, $($tt:tt)*) => {
        $v($($tt)*)
    };
    (_range, $start:literal..$end:literal) => {
        $crate::common::Range::new($start, $end - $start)
    }
}

#[cfg(not(any(feature = "block-id", feature = "line-number")))]
#[test]
fn it_works_in_block_phase() {
    let table: Vec<GroupedCases<_>> = vec![
        GroupedCases {
            group: "基础",
            cases: vec![
                case!("", [,], "",),
                case!(
                    "a:0/b:4", // “a” 的范围从 0 起， “b” 的范围从 4 起。
                    [
                        (EnterParagraph(..)),
                        (VerbatimEscaping(0..1)),
                        (NewLine(..)),
                        (Text(4..5)),
                        (ExitBlock(..)),
                    ],
                    "<p>a<br>b</p>",
                ),
                case!("", [(ThematicBreak(..)),], "<hr>",),
                case!(
                    "x1:0/x2:5/x3:10/x4:16/x5:22/x6:28",
                    [
                        (EnterHeading1(..)),
                        (EnterParagraph(..)),
                        (Text(0..2)),
                        (ExitBlock(..)),
                        (ExitBlock(..)),
                        (EnterHeading2(..)),
                        (EnterParagraph(..)),
                        (Text(5..7)),
                        (ExitBlock(..)),
                        (ExitBlock(..)),
                        (EnterHeading3(..)),
                        (EnterParagraph(..)),
                        (Text(10..12)),
                        (ExitBlock(..)),
                        (ExitBlock(..)),
                        (EnterHeading4(..)),
                        (EnterParagraph(..)),
                        (Text(16..18)),
                        (ExitBlock(..)),
                        (ExitBlock(..)),
                        (EnterHeading5(..)),
                        (EnterParagraph(..)),
                        (Text(22..24)),
                        (ExitBlock(..)),
                        (ExitBlock(..)),
                        (EnterHeading6(..)),
                        (EnterParagraph(..)),
                        (Text(28..30)),
                        (ExitBlock(..)),
                        (ExitBlock(..)),
                    ],
                    concat!(
                        "<h1><p>x1</p></h1>",
                        "<h2><p>x2</p></h2>",
                        "<h3><p>x3</p></h3>",
                        "<h4><p>x4</p></h4>",
                        "<h5><p>x5</p></h5>",
                        "<h6><p>x6</p></h6>",
                    ),
                ),
            ],
        },
        GroupedCases {
            group: "item-likes",
            cases: vec![
                case!(
                    "bq:0",
                    [
                        (EnterBlockQuote(..)),
                        (EnterParagraph(..)),
                        (Text(0..2)),
                        (ExitBlock(..)),
                        (ExitBlock(..)),
                    ],
                    "<blockquote><p>bq</p></blockquote>",
                ),
                case!(
                    "ol-a:0/ol-b:7/ul-a:14/ul-a.a:22",
                    [
                        (EnterOrderedList(..)),
                        (EnterListItem(..)),
                        (EnterParagraph(..)),
                        (Text(0..4)),
                        (ExitBlock(..)),
                        (ExitBlock(..)),
                        (EnterListItem(..)),
                        (EnterParagraph(..)),
                        (Text(7..11)),
                        (ExitBlock(..)),
                        (ExitBlock(..)),
                        (ExitBlock(..)),
                        (EnterUnorderedList(..)),
                        (EnterListItem(..)),
                        (EnterParagraph(..)),
                        (Text(14..18)),
                        (ExitBlock(..)),
                        (EnterUnorderedList(..)),
                        (EnterListItem(..)),
                        (EnterParagraph(..)),
                        (Text(22..28)),
                        (ExitBlock(..)),
                        (ExitBlock(..)),
                        (ExitBlock(..)),
                        (ExitBlock(..)),
                        (ExitBlock(..)),
                    ],
                    concat!(
                        "<ol><li><p>ol-a</p></li><li><p>ol-b</p></li></ol>",
                        "<ul><li><p>ul-a</p><ul><li><p>ul-a.a</p></li></ul></li></ul>",
                    ),
                ),
                case!(
                    "A:0/Aa:4/Ab:9/B:14",
                    [
                        (EnterDescriptionList(..)),
                        (EnterDescriptionTerm(..)),
                        (EnterParagraph(..)),
                        (Text(0..1)),
                        (ExitBlock(..)),
                        (ExitBlock(..)),
                        (EnterDescriptionDetails(..)),
                        (EnterParagraph(..)),
                        (Text(4..6)),
                        (ExitBlock(..)),
                        (ExitBlock(..)),
                        (EnterDescriptionDetails(..)),
                        (EnterParagraph(..)),
                        (Text(9..11)),
                        (ExitBlock(..)),
                        (ExitBlock(..)),
                        (EnterDescriptionTerm(..)),
                        (EnterParagraph(..)),
                        (Text(14..15)),
                        (ExitBlock(..)),
                        (ExitBlock(..)),
                        (ExitBlock(..)),
                    ],
                    "<dl><dt><p>A</p></dt><dd><p>Aa</p></dd><dd><p>Ab</p></dd><dt><p>B</p></dt></dl>",
                ),
            ]
        },
        GroupedCases {
            group: "代码块",
            cases: vec![
                case!(
                    "info:0/code:7",
                    [
                        (EnterCodeBlock(..)),
                        (Text(0..4)),
                        (Separator()),
                        (Text(7..11)),
                        (ExitBlock(..)),
                    ],
                    r#"<x-code-block info-string="info">code</x-code-block>"#,
                ),
            ],
        },
    ];

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

// TODO: test for feature "block-id".

#[allow(dead_code)]
struct CaseOptions {
    #[cfg(feature = "block-id")]
    with_block_id: bool,
}

#[allow(dead_code)]
struct Case {
    input: &'static str,
    input_events: Vec<BlendEvent>,
    expected: &'static str,

    #[allow(dead_code)]
    options: CaseOptions,
}
impl test_utils::Case for Case {
    fn assert_ok(&self) {
        let opts = NewHtmlRendererOptoins {
            initial_output_string_capacity: 0,
            #[cfg(feature = "block-id")]
            with_block_id: self.options.with_block_id,
        };
        let renderer = HtmlRenderer::new(self.input.as_bytes(), opts);
        let actual = renderer.render(self.input_events.clone().into_iter());

        assert_eq!(self.expected, actual);
    }

    fn input(&self) -> String {
        format!("{:?}", self.input_events)
    }
}
