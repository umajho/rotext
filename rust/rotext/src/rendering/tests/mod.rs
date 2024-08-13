#![cfg(test)]

mod support;

use support::{case, run_cases};

#[allow(unused_imports)]
use crate::test_support::{self, report_panicked_cases, FailedCase, GroupedCases};

use super::*;

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
                    "info:0/code:7/line 2:14",
                    [
                        (EnterCodeBlock(..)),
                        (Text(0..4)),
                        (IndicateCodeBlockCode()),
                        (Text(7..11)),
                        (NewLine(..)),
                        (Text(14..20)),
                        (ExitBlock(..)),
                    ],
                    r#"<x-code-block info-string="info" content="code&#10;line 2"></x-code-block>"#,
                ),
            ],
        },
        GroupedCases {
            group: "表格",
            cases: vec![
                case!(
                    "",
                    [
                        (EnterTable(..)),
                        (ExitBlock(..)),
                    ],
                    "<table></table>",
                ),
                case!(
                    "",
                    [
                        (EnterTable(..)),
                        (IndicateTableRow()),
                        (ExitBlock(..)),
                    ],
                    "<table><tr></tr></table>",
                ),
                case!(
                    "",
                    [
                        (EnterTable(..)),
                        (IndicateTableDataCell()),
                        (ExitBlock(..)),
                    ],
                    "<table><tr><td></td></tr></table>",
                ),
                case!(
                    "",
                    [
                        (EnterTable(..)),
                        (IndicateTableRow()),
                        (IndicateTableDataCell()),
                        (ExitBlock(..)),
                    ],
                    "<table><tr><td></td></tr></table>",
                ),
                case!(
                    "",
                    [
                        (EnterTable(..)),
                        (IndicateTableHeaderCell()),
                        (ExitBlock(..)),
                    ],
                    "<table><tr><th></th></tr></table>",
                ),
                case!(
                    "",
                    [
                        (EnterTable(..)),
                        (IndicateTableRow()),
                        (IndicateTableHeaderCell()),
                        (ExitBlock(..)),
                    ],
                    "<table><tr><th></th></tr></table>",
                ),
                case!(
                    "",
                    [
                        (EnterTable(..)),
                        (IndicateTableRow()),
                        (IndicateTableRow()),
                        (ExitBlock(..)),
                    ],
                    concat!(
                        "<table><tr></tr><tr></tr></table>",
                    ),
                ),
                case!(
                    "",
                    [
                        (EnterTable(..)),
                        (IndicateTableDataCell()),
                        (IndicateTableDataCell()),
                        (ExitBlock(..)),
                    ],
                    concat!(
                        "<table><tr><td></td><td></td></tr></table>",
                    ),
                ),
                case!(
                    "data",
                    [
                        (EnterTable(..)),
                        (EnterParagraph(..)),
                        (Text(0..4)),
                        (ExitBlock(..)),
                        (ExitBlock(..)),
                    ],
                    "<table><tr><td><p>data</p></td></tr></table>",
                ),
                case!(
                    "data",
                    [
                        (EnterTable(..)),
                        (IndicateTableHeaderCell()),
                        (EnterParagraph(..)),
                        (Text(0..4)),
                        (ExitBlock(..)),
                        (ExitBlock(..)),
                    ],
                    "<table><tr><th><p>data</p></th></tr></table>",
                ),
                case!(
                    "ABCD",
                    [
                        (EnterTable(..)),
                        (IndicateTableHeaderCell()),
                        (EnterParagraph(..)),
                        (Text(0..1)),
                        (ExitBlock(..)),
                        (IndicateTableHeaderCell()),
                        (EnterParagraph(..)),
                        (Text(1..2)),
                        (ExitBlock(..)),
                        (IndicateTableRow()),
                        (EnterParagraph(..)),
                        (Text(2..3)),
                        (ExitBlock(..)),
                        (IndicateTableDataCell()),
                        (EnterParagraph(..)),
                        (Text(3..4)),
                        (ExitBlock(..)),
                        (ExitBlock(..)),
                    ],
                    concat!(
                        "<table>",
                        "<tr><th><p>A</p></th><th><p>B</p></th></tr>",
                        "<tr><td><p>C</p></td><td><p>D</p></td></tr>",
                        "</table>",
                    ),
                ),
            ]
        },
        GroupedCases {
            group: "表格>captions",
            cases: vec![
                case!(
                    "CAPTION",
                    [
                        (EnterTable(..)),
                        (IndicateTableCaption()),
                        (EnterParagraph(..)),
                        (Text(0..7)),
                        (ExitBlock(..)),
                        (ExitBlock(..)),
                    ],
                    "<table><caption><p>CAPTION</p></caption></table>",
                ),
                case!(
                    "CAPTION",
                    [
                        (EnterTable(..)),
                        (IndicateTableCaption()),
                        (EnterParagraph(..)),
                        (Text(0..7)),
                        (ExitBlock(..)),
                        (IndicateTableRow()),
                        (ExitBlock(..)),
                    ],
                    "<table><caption><p>CAPTION</p></caption><tr></tr></table>",
                ),
                case!(
                    "CAPTION",
                    [
                        (EnterTable(..)),
                        (IndicateTableCaption()),
                        (EnterParagraph(..)),
                        (Text(0..7)),
                        (ExitBlock(..)),
                        (IndicateTableRow()),
                        (IndicateTableDataCell()),
                        (ExitBlock(..)),
                    ],
                    "<table><caption><p>CAPTION</p></caption><tr><td></td></tr></table>",
                ),
                case!(
                    "CAPTION",
                    [
                        (EnterTable(..)),
                        (IndicateTableCaption()),
                        (EnterParagraph(..)),
                        (Text(0..7)),
                        (ExitBlock(..)),
                        (IndicateTableHeaderCell()),
                        (ExitBlock(..)),
                    ],
                    "<table><caption><p>CAPTION</p></caption><tr><th></th></tr></table>",
                ),
                case!(
                    "CAPTION",
                    [
                        (EnterTable(..)),
                        (IndicateTableCaption()),
                        (EnterParagraph(..)),
                        (Text(0..7)),
                        (ExitBlock(..)),
                        (IndicateTableDataCell()),
                        (ExitBlock(..)),
                    ],
                    "<table><caption><p>CAPTION</p></caption><tr><td></td></tr></table>",
                ),
            ],
        },
        GroupedCases {
            group: "表格>嵌套",
            cases: vec![
                case!(
                    "CAPTION:0/DATA:10",
                    [
                        (EnterTable(..)),
                        (IndicateTableCaption()),
                        (EnterTable(..)),
                        (IndicateTableCaption()),
                        (EnterParagraph(..)),
                        (Text(0..7)),
                        (ExitBlock(..)),
                        (IndicateTableDataCell()),
                        (EnterParagraph(..)),
                        (Text(10..14)),
                        (ExitBlock(..)),
                        (ExitBlock(..)),
                        (IndicateTableDataCell()),
                        (EnterTable(..)),
                        (IndicateTableCaption()),
                        (EnterParagraph(..)),
                        (Text(0..7)),
                        (ExitBlock(..)),
                        (ExitBlock(..)),
                        (ExitBlock(..)),
                    ],
                    concat!(
                        "<table><caption>",
                        "<table><caption><p>CAPTION</p></caption><tr><td><p>DATA</p></td></tr></table>",
                        "</caption><tr><td>",
                        "<table><caption><p>CAPTION</p></caption></table>",
                        "</td></tr></table>",
                    ),
                ),
            ],
        },
        GroupedCases {
            group: "XSS",
            cases: vec![
                case!(
                    r#"<script>"#,
                    [
                        (EnterParagraph(..)),
                        (Text(0..8)),
                        (ExitBlock(..)),
                    ],
                    r#"<p>&lt;script></p>"#,
                ),
                case!(
                    r#"">"#,
                    [
                        (EnterCodeBlock(..)),
                        (Text(0..2)),
                        (IndicateCodeBlockCode()),
                        (Text(0..2)),
                        (ExitBlock(..)),
                    ],
                    r#"<x-code-block info-string="&quot;>" content="&quot;>"></x-code-block>"#,
                ),
            ],
        },
    ];

    run_cases(table);
}

#[cfg(all(feature = "block-id", feature = "line-number"))]
#[test]
fn it_works_with_block_id() {
    let table: Vec<GroupedCases<_>> = vec![GroupedCases {
        group: "基础",
        cases: vec![
            case!(
                @with_id,
                "",
                [(ThematicBreak(.., id = 1, ln = 1)),],
                r#"<hr data-block-id="1">"#,
            ),
            case!(
                @with_id,
                "foo",
                [
                    (EnterParagraph(.., id = 1)),
                    (Text(0..3)),
                    (ExitBlock(.., id = 1, lns = 1..1)),
                ],
                r#"<p data-block-id="1">foo</p>"#,
            ),
            case!(
                @with_id,
                "foo:0/bar:6",
                [
                    (EnterTable(.., id = 1)),
                    (IndicateTableHeaderCell()),
                    (EnterParagraph(.., id = 2)),
                    (Text(0..3)),
                    (ExitBlock(.., id = 2, lns = 1..1)),
                    (IndicateTableRow()),
                    (EnterParagraph(.., id = 3)),
                    (Text(6..9)),
                    (ExitBlock(.., id = 3, lns = 1..1)),
                    (ExitBlock(.., id = 1, lns = 1..1)),
                ],
                concat!(
                    r#"<table data-block-id="1">"#,
                    r#"<tr><th><p data-block-id="2">foo</p></th></tr>"#,
                    r#"<tr><td><p data-block-id="3">bar</p></td></tr>"#,
                    "</table>"
                ),
            ),
        ],
    }];

    run_cases(table);
}
