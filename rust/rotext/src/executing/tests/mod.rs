mod fixtures;
mod support;

use support::{case, run_cases};

#[allow(unused_imports)]
use rotext_internal_test::support::{FailedCase, GroupedCases, report_panicked_cases};

use super::*;

#[test]
fn it_works_in_block_phase_for_simple_events() {
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
                case!(
                    "a\nb",
                    [
                        (EnterParagraph(..)),
                        (VerbatimEscaping(0..3)),
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
            ],
        },
        GroupedCases {
            group: "代码块",
            cases: vec![case!(
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
            )],
        },
        GroupedCases {
            group: "表格",
            cases: vec![
                case!("", [(EnterTable(..)), (ExitBlock(..)),], "<table></table>",),
                case!(
                    "",
                    [(EnterTable(..)), (IndicateTableRow()), (ExitBlock(..)),],
                    "<table><tr></tr></table>",
                ),
                case!(
                    "",
                    [(EnterTable(..)), (IndicateTableDataCell()), (ExitBlock(..)),],
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
                    concat!("<table><tr></tr><tr></tr></table>",),
                ),
                case!(
                    "",
                    [
                        (EnterTable(..)),
                        (IndicateTableDataCell()),
                        (IndicateTableDataCell()),
                        (ExitBlock(..)),
                    ],
                    concat!("<table><tr><td></td><td></td></tr></table>",),
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
            ],
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
            cases: vec![case!(
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
            )],
        },
        GroupedCases {
            group: "Wiki链接",
            cases: vec![case!(
                "ADDR:0/title:10",
                [
                    (EnterParagraph(..)),
                    (EnterWikiLink(0..4)),
                    (Text(7..12)),
                    (@inline ExitInline(..)),
                    (ExitBlock(..)),
                ],
                r#"<p><x-wiki-link address="ADDR">title</x-wiki-link></p>"#,
            )],
        },
        GroupedCases {
            group: "XSS",
            cases: vec![
                case!(
                    r#"<script>"#,
                    [(EnterParagraph(..)), (Text(0..8)), (ExitBlock(..)),],
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

#[cfg(feature = "block-id")]
#[test]
fn it_works_in_block_phase_for_simple_events_with_feature_block_id() {
    let table: Vec<GroupedCases<_>> = vec![GroupedCases {
        group: "基础",
        cases: vec![
            case!(
                @with_id,
                "",
                [(ThematicBreak(.., id = 1)),],
                r#"<hr data-block-id="1">"#,
            ),
            case!(
                @with_id,
                "foo",
                [
                    (EnterParagraph(.., id = 1)),
                    (Text(0..3)),
                    (ExitBlock(.., id = 1)),
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
                    (ExitBlock(.., id = 2)),
                    (IndicateTableRow()),
                    (EnterParagraph(.., id = 3)),
                    (Text(6..9)),
                    (ExitBlock(.., id = 3)),
                    (ExitBlock(.., id = 1)),
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

#[test]
fn it_works_in_block_phase_for_events_involving_calls() {
    let table: Vec<GroupedCases<_>> = vec![
        GroupedCases {
            group: "调用>扩展",
            cases: vec![
                case!(
                    "AllOptional",
                    [(EnterCallOnExtension(block, 0..11)), (ExitBlock(..)),],
                    r#"<all-optional></all-optional>"#,
                ),
                case!(
                    "AllOptional:0/1_content:14",
                    [
                        (EnterCallOnExtension(block, 0..11)),
                        (IndicateCallNormalArgument()),
                        (EnterParagraph(..)),
                        (Text(14..23)),
                        (ExitBlock(..)),
                        (ExitBlock(..)),
                    ],
                    r#"<all-optional><p>1_content</p></all-optional>"#,
                ),
                case!(
                    "AllOptional:0/first_content:14",
                    [
                        (EnterCallOnExtension(block, 0..11)),
                        (IndicateCallVerbatimArgument()),
                        (Text(14..27)),
                        (ExitBlock(..)),
                    ],
                    r#"<all-optional first="first_content"></all-optional>"#,
                ),
                case!(
                    "AllOptional:0/1:14/1_content:19",
                    [
                        (EnterCallOnExtension(block, 0..11)),
                        (IndicateCallNormalArgument(14..15)),
                        (EnterParagraph(..)),
                        (Text(19..28)),
                        (ExitBlock(..)),
                        (ExitBlock(..)),
                    ],
                    r#"<all-optional><p>1_content</p></all-optional>"#,
                ),
                case!(
                    "AllOptional:0/1_content:14/2_content:27",
                    [
                        (EnterCallOnExtension(block, 0..11)),
                        (IndicateCallNormalArgument()),
                        (EnterParagraph(..)),
                        (Text(14..23)),
                        (ExitBlock(..)),
                        (IndicateCallNormalArgument()),
                        (EnterParagraph(..)),
                        (Text(27..36)),
                        (ExitBlock(..)),
                        (ExitBlock(..)),
                    ],
                    r#"<all-optional><p>1_content</p><div slot="second"><p>2_content</p></div></all-optional>"#,
                ),
                case!(
                    "AllOptional:0/1_content:14/foo:27/foo_content:34",
                    [
                        (EnterCallOnExtension(block, 0..11)),
                        (IndicateCallNormalArgument()),
                        (EnterParagraph(..)),
                        (Text(14..23)),
                        (ExitBlock(..)),
                        (IndicateCallNormalArgument(27..30)),
                        (EnterParagraph(..)),
                        (Text(34..45)),
                        (ExitBlock(..)),
                        (ExitBlock(..)),
                    ],
                    r#"<all-optional><p>1_content</p><div slot="foo"><p>foo_content</p></div></all-optional>"#,
                ),
                case!(
                    "AllOptional:0/bar:14/bar_content:21",
                    [
                        (EnterCallOnExtension(block, 0..11)),
                        (IndicateCallVerbatimArgument(14..17)),
                        (Text(21..32)),
                        (ExitBlock(..)),
                    ],
                    r#"<all-optional bar="bar_content"></all-optional>"#,
                ),
                case!(
                    "AllOptional:0/bar:14/bar_content:21/baz:36/baz_content:43",
                    [
                        (EnterCallOnExtension(block, 0..11)),
                        (IndicateCallVerbatimArgument(14..17)),
                        (Text(21..32)),
                        (IndicateCallVerbatimArgument(36..39)),
                        (Text(43..54)),
                        (ExitBlock(..)),
                    ],
                    r#"<all-optional bar="bar_content" baz="baz_content"></all-optional>"#,
                ),
                case!(
                    "AllOptional:0/1_content:14/foo:27/foo_content:34/bar:49/bar_content:56/baz:71/baz_content:78",
                    [
                        (EnterCallOnExtension(block, 0..11)),
                        (IndicateCallNormalArgument()),
                        (EnterParagraph(..)),
                        (Text(14..23)),
                        (ExitBlock(..)),
                        (IndicateCallNormalArgument(27..30)),
                        (EnterParagraph(..)),
                        (Text(34..45)),
                        (ExitBlock(..)),
                        (IndicateCallVerbatimArgument(49..52)),
                        (Text(56..67)),
                        (IndicateCallVerbatimArgument(71..74)),
                        (Text(78..89)),
                        (ExitBlock(..)),
                    ],
                    r#"<all-optional bar="bar_content" baz="baz_content"><p>1_content</p><div slot="foo"><p>foo_content</p></div></all-optional>"#,
                ),
                case!(
                    "AllOptional:0/bar:14/bar_content:21/1_content:36/baz:49/baz_content:56/foo:71/foo_content:78",
                    [
                        (EnterCallOnExtension(block, 0..11)),
                        (IndicateCallVerbatimArgument(14..17)),
                        (Text(21..32)),
                        (IndicateCallNormalArgument()),
                        (EnterParagraph(..)),
                        (Text(36..45)),
                        (ExitBlock(..)),
                        (IndicateCallVerbatimArgument(49..52)),
                        (Text(56..67)),
                        (IndicateCallNormalArgument(71..74)),
                        (EnterParagraph(..)),
                        (Text(78..89)),
                        (ExitBlock(..)),
                        (ExitBlock(..)),
                    ],
                    r#"<all-optional bar="bar_content" baz="baz_content"><p>1_content</p><div slot="foo"><p>foo_content</p></div></all-optional>"#,
                ),
            ],
        },
        GroupedCases {
            group: "调用>扩展>必要参数",
            cases: vec![
                case!(
                    "SomeNormalRequired",
                    [(EnterCallOnExtension(block, 0..18)), (ExitBlock(..)),],
                    r#"<x-block-call-error call-type="extension" call-name="SomeNormalRequired" error-type="BadParameters" error-value="!1;"></x-block-call-error>"#,
                ),
                case!(
                    "SomeNormalRequired:0/test:21",
                    [
                        (EnterCallOnExtension(block, 0..18)),
                        (IndicateCallNormalArgument()),
                        (EnterParagraph(..)),
                        (Text(21..25)),
                        (ExitBlock(..)),
                        (ExitBlock(..)),
                    ],
                    r#"<some-normal-required><p>test</p></some-normal-required>"#,
                ),
                case!(
                    "SomeVerbatimRequired",
                    [(EnterCallOnExtension(block, 0..20)), (ExitBlock(..)),],
                    r#"<x-block-call-error call-type="extension" call-name="SomeVerbatimRequired" error-type="BadParameters" error-value=";!bar"></x-block-call-error>"#,
                ),
                case!(
                    "SomeVerbatimRequired:0/bar:23",
                    [
                        (EnterCallOnExtension(block, 0..20)),
                        (IndicateCallVerbatimArgument(23..26)),
                        (ExitBlock(..)),
                    ],
                    r#"<some-verbatim-required bar=""></some-verbatim-required>"#,
                ),
            ],
        },
        GroupedCases {
            group: "调用>扩展>变体",
            cases: vec![case!(
                "WithVariant",
                [(EnterCallOnExtension(block, 0..11)), (ExitBlock(..)),],
                r#"<with-variant variant="var"></with-variant>"#,
            )],
        },
        GroupedCases {
            group: "调用>扩展>扩展别名",
            cases: vec![case!(
                "Alias",
                [(EnterCallOnExtension(block, 0..5)), (ExitBlock(..)),],
                r#"<all-optional></all-optional>"#,
            )],
        },
        GroupedCases {
            group: "调用>扩展>参数别名",
            cases: vec![
                case!(
                    "AllOptional:0/alias:14/foo:23",
                    [
                        (EnterCallOnExtension(block, 0..11)),
                        (IndicateCallNormalArgument(14..19)),
                        (EnterParagraph(..)),
                        (Text(23..26)),
                        (ExitBlock(..)),
                        (ExitBlock(..)),
                    ],
                    r#"<all-optional><p>foo</p></all-optional>"#,
                ),
                case!(
                    "AllOptional:0/alias:14/foo:23",
                    [
                        (EnterCallOnExtension(block, 0..11)),
                        (IndicateCallNormalArgument(14..19)),
                        (EnterParagraph(..)),
                        (Text(23..26)),
                        (ExitBlock(..)),
                        (ExitBlock(..)),
                    ],
                    r#"<all-optional><p>foo</p></all-optional>"#,
                ),
                case!(
                    "AllOptional:0/alias:14/foo:23",
                    [
                        (EnterCallOnExtension(block, 0..11)),
                        (IndicateCallNormalArgument()),
                        (EnterParagraph(..)),
                        (Text(23..26)),
                        (ExitBlock(..)),
                        (IndicateCallNormalArgument(14..19)),
                        (EnterParagraph(..)),
                        (Text(23..26)),
                        (ExitBlock(..)),
                        (ExitBlock(..)),
                    ],
                    r#"<x-block-call-error call-type="extension" call-name="AllOptional" error-type="BadParameters" error-value="=1;"></x-block-call-error>"#,
                ),
                case!(
                    "AllOptional:0/alias:14/foo:23",
                    [
                        (EnterCallOnExtension(block, 0..11)),
                        (IndicateCallNormalArgument(14..19)),
                        (EnterParagraph(..)),
                        (Text(23..26)),
                        (ExitBlock(..)),
                        (IndicateCallNormalArgument()),
                        (EnterParagraph(..)),
                        (Text(23..26)),
                        (ExitBlock(..)),
                        (ExitBlock(..)),
                    ],
                    r#"<x-block-call-error call-type="extension" call-name="AllOptional" error-type="BadParameters" error-value="=1;"></x-block-call-error>"#,
                ),
                case!(
                    "AllOptional:0/alias:14/foo:23/1:30",
                    [
                        (EnterCallOnExtension(block, 0..11)),
                        (IndicateCallNormalArgument(30..31)),
                        (EnterParagraph(..)),
                        (Text(23..26)),
                        (ExitBlock(..)),
                        (IndicateCallNormalArgument(14..19)),
                        (EnterParagraph(..)),
                        (Text(23..26)),
                        (ExitBlock(..)),
                        (ExitBlock(..)),
                    ],
                    r#"<x-block-call-error call-type="extension" call-name="AllOptional" error-type="BadParameters" error-value="=1;"></x-block-call-error>"#,
                ),
            ],
        },
    ];

    run_cases(table);
}

#[cfg(feature = "block-id")]
#[test]
fn it_works_in_block_phase_for_events_involving_calls_with_feature_block_id() {
    let table: Vec<GroupedCases<_>> = vec![GroupedCases {
        group: "调用>扩展",
        cases: vec![case!(
            @with_id,
            "Alias",
            [(EnterCallOnExtension(block, 0..5, id = 1)), (ExitBlock(..)),],
            r#"<all-optional data-block-id="1"></all-optional>"#,
        )],
    }];

    run_cases(table);
}

#[test]
fn it_works_in_inline_phase() {
    let table: Vec<GroupedCases<_>> = vec![
        GroupedCases {
            group: "引用链接",
            cases: vec![case!(
                "TP.abc",
                [(RefLink(0..6)),],
                r#"<x-ref-link address="TP.abc"></x-ref-link>"#,
            )],
        },
        GroupedCases {
            group: "Dicexp",
            cases: vec![case!(
                "d100",
                [(Dicexp(0..4)),],
                r#"<x-dicexp code="d100"></x-dicexp>"#,
            )],
        },
        GroupedCases {
            group: "加粗强调与删除线",
            cases: vec![
                case!(
                    "foo",
                    [
                        (@inline EnterStrong(..)),
                        (Text(0..3)),
                        (@inline ExitInline(..)),
                    ],
                    r#"<strong>foo</strong>"#,
                ),
                case!(
                    "foo",
                    [
                        (@inline EnterStrikethrough(..)),
                        (Text(0..3)),
                        (@inline ExitInline(..)),
                    ],
                    r#"<s>foo</s>"#,
                ),
                case!(
                    "foo:0/bar:6/baz:12",
                    [
                        (@inline EnterStrong(..)),
                        (@inline EnterStrikethrough(..)),
                        (Text(0..3)),
                        (@inline ExitInline(..)),
                        (@inline EnterStrong(..)),
                        (Text(6..9)),
                        (@inline ExitInline(..)),
                        (Text(12..15)),
                        (@inline ExitInline(..)),
                    ],
                    r#"<strong><s>foo</s><strong>bar</strong>baz</strong>"#,
                ),
            ],
        },
        GroupedCases {
            group: "XSS",
            cases: vec![case!(
                r#"""#,
                [(Dicexp(0..1)),],
                r#"<x-dicexp code="&quot;"></x-dicexp>"#,
            )],
        },
    ];

    run_cases(table);
}

#[test]
fn it_works_in_inline_phase_for_events_involving_calls() {
    let table: Vec<GroupedCases<_>> = vec![GroupedCases {
        group: "调用>扩展",
        cases: vec![case!(
            "AllOptional",
            [(EnterCallOnExtension(inline, 0..11)), (@inline ExitInline(..)),],
            r#"<i-all-optional></i-all-optional>"#,
        )],
    }];

    // 由于块级阶段与行内阶段目前渲染调用使用同样的实现，这里就不重复测试了。
    // TODO: 也许应该实现一个函数用于同时生成这方面块级阶段与行内阶段的测试用例。

    run_cases(table);
}

#[cfg(feature = "block-id")]
#[test]
fn it_works_in_inline_phase_for_events_involving_calls_with_feature_block_id() {
    let table: Vec<GroupedCases<_>> = vec![GroupedCases {
        group: "调用>扩展",
        cases: vec![case!(
            @with_id,
            "Alias",
            [(EnterCallOnExtension(inline, 0..5, id = 1)), (@inline ExitInline(..)),],
            r#"<i-all-optional></i-all-optional>"#,
        )],
    }];

    run_cases(table);
}
