import { describe, expect, it } from "vitest";

import { create, createDocument, type Document } from "@rotext/nodes";

import { parse, ParseOptions } from "../lib";

interface Case {
  input: string;
  expected: Document["slot"];
}

function assertOk(
  input: string,
  expected: Document["slot"] | null,
  opts?: ParseOptions,
) {
  const output = parse(input, opts);
  if (expected) {
    expect(output).toStrictEqual(createDocument(expected));
  }
}

function theseCasesAreOk(cases: Case[], opts?: ParseOptions) {
  for (const [i, theCase] of cases.entries()) {
    it(`case ${i + 1}: ${JSON.stringify(theCase.input)} ok`, () => {
      assertOk(theCase.input, theCase.expected, opts);
    });
  }
}

describe("解析", () => {
  describe("行内元素", () => {
    describe("文本", () => {
      describe("一般内容", () => {
        theseCasesAreOk([
          { input: "foo", expected: [create.P(["foo"])] },
        ]);
      });
      describe("转义（基础）", () => {
        // NOTE: 这里没有涉及的元素的测试由测试对应元素的地方负责

        describe("于原先的单行块元素开头", () => {
          theseCasesAreOk([
            { input: String.raw`\---`, expected: [create.P(["---"])] },
          ]);
        });
        describe("于行内（基础：对自身的转义及对无特殊意义字符的转义）", () => {
          theseCasesAreOk([
            { input: String.raw`\\`, expected: [create.P(["\\"])] },
            { input: String.raw`\a`, expected: [create.P(["a"])] },
            { input: "\\", expected: [create.P(["\\"])] },
            { input: "a\\", expected: [create.P(["a\\"])] },
          ]);
        });
      });
      describe("多行", () => {
        describe("`breaks` 选项为假时，输入文本中的单次换行只视为空格", () => {
          theseCasesAreOk([
            {
              input: "foo\nbar",
              expected: [create.P(["foo bar"])],
            },
          ], { softBreakAs: "space" });
        });
        describe("`breaks` 选项为真时，输入文本中的单次换行视为换行", () => {
          theseCasesAreOk([
            {
              input: "foo\nbar",
              expected: [create.P(["foo", create.br(), "bar"])],
            },
            {
              input: "foo\n['bar']\nbaz",
              expected: [
                create.P([
                  "foo",
                  create.br(),
                  create.em("strong", ["bar"]),
                  create.br(),
                  "baz",
                ]),
              ],
            },
          ], { softBreakAs: "br" });
        });
      });
    });
    describe("引用链接", () => {
      describe("能正确解析格式正确的", () => {
        theseCasesAreOk(
          [
            "TP.42",
            ...["TP.abc" /*, "abc", "~" */],
            ...["TP.abc#123" /*, "abc#123", "#123", "#" */],
            ...["TP.abc.def" /*, "abc.def", "~.def" */],
            ...["TP.abc.def#456" /*, "abc.def#456", "~.def#456" */],
          ]
            .map((input) => ({
              input: `>>${input}`,
              expected: [create.P([create.ref_link(input)])],
            })),
        );
      });
      describe("忽略目前格式被禁用的", () => {
        theseCasesAreOk(
          [
            ...[
              ...["abc", "~"],
              ...["abc#123", "#123", "#"],
              ...["abc.def", "~.def"],
              ...["abc.def#456", "~.def#456"],
            ].map((input) => ({
              input: `>>${input}`,
              expected: [create.P([`>>${input}`])],
            })),
          ],
        );
      });
      describe("忽略格式不正确的", () => {
        theseCasesAreOk(
          [
            ...[
              ...["", "42"],
              ...["TP.~", "TP.#123", "TP.#"],
              ...[".def", "TP.~.def"],
              ...["#123.456"],
            ].map((input) => ({
              input: `>>${input}`,
              expected: [create.P([`>>${input}`])],
            })),
            {
              input: ">>TP.abc#123.def",
              expected: [create.P([create.ref_link("TP.abc#123"), ".def"])],
            },
          ],
        );
      });
    });
    describe("行内代码文本", () => {
      describe("能正确解析", () => {
        theseCasesAreOk([
          ...["[`foo`]"].map((input) => ({
            input,
            expected: [create.P([create.code("foo")])],
          })),
          {
            input: "[`['foo']`]",
            expected: [create.P([create.code("['foo']")])],
          },
        ]);
      });
      describe("贪婪（？）", () => {
        theseCasesAreOk([
          {
            input: "[`[`foo`]`]",
            expected: [create.P(["[`", create.code("foo"), "`]"])],
          },
        ]);
      });
      describe("目前即使内容为空也会创建", () => {
        theseCasesAreOk([
          { input: "[``]", expected: [create.P([create.code("")])] },
        ]);
      });
      describe("只存在 “``” 转义", () => {
        theseCasesAreOk([
          { input: "[````]", expected: [create.P([create.code("`")])] },
          { input: "[`\\a`]", expected: [create.P([create.code("\\a")])] },
          { input: "[`\\`]", expected: [create.P([create.code("\\")])] },
        ]);
      });
    });
    describe("文本相关样式", () => {
      describe("能正确解析", () => {
        theseCasesAreOk([
          ...["['foo']"].map((input) => ({
            input,
            expected: [create.P([create.em("strong", ["foo"])])],
          })),
          // ...["[/foo/]"].map((input) => ({
          //   input,
          //   expected: [create.P([create.em(null, ["foo"])])],
          // })),
          // ...["[_foo_]"].map((input) => ({
          //   input,
          //   expected: [create.P([create.u(["foo"])])],
          // })),
          ...["[~foo~]"].map((input) => ({
            input,
            expected: [create.P([create.s(["foo"])])],
          })),
          // ...["[.foo.]"].map((input) => ({
          //   input,
          //   expected: [create.P([create.em("dotted", ["foo"])])],
          // })),
        ]);
      });
      describe("目前即使内容为空也会创建", () => {
        theseCasesAreOk([
          { input: "['']", expected: [create.P([create.em("strong", [])])] },
        ]);
      });
      describe("内部可以嵌套行内元素", () => {
        theseCasesAreOk([
          {
            input: "['[~foo~]']",
            expected: [create.P([create.em("strong", [create.s(["foo"])])])],
          },
          {
            input: "['[`foo`]']",
            expected: [create.P([create.em("strong", [create.code("foo")])])],
          },
        ]);
      });
      describe("只含部分标记时视为一般文本", () => {
        theseCasesAreOk([
          { input: "[]", expected: [create.P(["[]"])] },
          { input: "[foo]", expected: [create.P(["[foo]"])] },
          { input: "['foo", expected: [create.P(["['foo"])] },
        ]);
      });
      describe("转义", () => {
        theseCasesAreOk([
          ...["\\['foo']", "[\\'foo']", "['foo\\']", "['foo'\\]"].map(
            (input) => ({ input, expected: [create.P(["['foo']"])] }),
          ),
          {
            input: String.raw`['\foo']`,
            expected: [create.P([create.em("strong", ["foo"])])],
          },
          {
            input: String.raw`['\['foo']`,
            expected: [create.P([create.em("strong", ["['foo"])])],
          },
          {
            input: String.raw`['foo\']']`,
            expected: [create.P([create.em("strong", ["foo']"])])],
          },
        ]);
      });
    });
    describe("注音", () => {
      describe("能正确解析", () => {
        theseCasesAreOk([
          {
            input: "[foo(bar)]",
            expected: [
              create.P([create.ruby(["foo"], ["bar"], ["(", ")"])]),
            ],
          },
          {
            input: "[测(•)][试(•)]",
            expected: [
              create.P([
                create.ruby(["测"], ["•"], ["(", ")"]),
                create.ruby(["试"], ["•"], ["(", ")"]),
              ]),
            ],
          },
        ]);
      });
      describe("支持全角 fallback 括号", () => {
        theseCasesAreOk([
          {
            input: "[测试（test）]",
            expected: [
              create.P([
                create.ruby(["测试"], ["test"], ["（", "）"]),
              ]),
            ],
          },
        ]);
      });
      describe("base 和 text 部分都支持行内元素", () => {
        theseCasesAreOk([
          {
            input: "[[`foo`](bar)]",
            expected: [
              create.P([
                create.ruby([create.code("foo")], ["bar"], ["(", ")"]),
              ]),
            ],
          },
          {
            input: "[foo([`bar`])]",
            expected: [
              create.P([
                create.ruby(["foo"], [create.code("bar")], ["(", ")"]),
              ]),
            ],
          },
          {
            input: "[1 [`2`] 3(4 [5(6)] 7)]",
            expected: [
              create.P([
                create.ruby(
                  ["1 ", create.code("2"), " 3"],
                  ["4 ", create.ruby(["5"], ["6"], ["(", ")"]), " 7"],
                  ["(", ")"],
                ),
              ]),
            ],
          },
        ]);
      });
    });
  });
  describe("块级元素", () => {
    describe("段落", () => {
      describe("两次换行开启新的段落", () => {
        theseCasesAreOk([
          {
            input: "foo\n\nbar",
            expected: [create.P(["foo"]), create.P(["bar"])],
          },
        ]);
      });
      describe("非段落间的空行不会产生空段落", () => {
        theseCasesAreOk(["\n", "\n\n", "\n\n\n", "\n\n\n\n"].flatMap((brs) => [
          { input: `${brs}---`, expected: [create.THEMATIC_BREAK()] },
          { input: `---${brs}`, expected: [create.THEMATIC_BREAK()] },
          {
            input: `---${brs}---`,
            expected: [create.THEMATIC_BREAK(), create.THEMATIC_BREAK()],
          },
        ]));
      });
    });
    describe("Thematic Break", () => {
      describe("能正确解析", () => {
        theseCasesAreOk([
          { input: "---", expected: [create.THEMATIC_BREAK()] },
        ]);
      });
    });
    describe("标题（Heading）", () => {
      describe("能正确解析", () => {
        const boringCases: Case[] = [];
        for (let n = 1; n <= 6; n++) {
          const signs = "=".repeat(n);
          boringCases.push({
            input: `${signs} foo ${signs}`,
            expected: [create.H(n as any, ["foo"])],
          });
        }

        theseCasesAreOk([
          ...boringCases,
          {
            input: "foo\n\n= bar =\nbaz",
            expected: [
              create.P(["foo"]),
              create.H(1, ["bar"]),
              create.P(["baz"]),
            ],
          },
        ]);
      });
      describe("内容与前后之间必须有空白", () => {
        theseCasesAreOk([{
          input: "==foo==",
          expected: [create.P(["==foo=="])],
        }]);
      });
      describe("前后的标记必须配对", () => {
        theseCasesAreOk([
          { input: "== foo =", expected: [create.P(["== foo ="])] },
          { input: "= foo ==", expected: [create.P(["= foo =="])] },
          {
            input: "======= foo =======",
            expected: [create.P(["======= foo ======="])],
          },
        ]);
      });
      describe("内部可以有行内元素", () => {
        theseCasesAreOk([
          {
            input: "== ['foo'] ==",
            expected: [create.H(2, [create.em("strong", ["foo"])])],
          },
        ]);
      });
    });
    describe("容器", () => {
      describe("单项", () => {
        theseCasesAreOk([
          { input: "> foo", expected: [create.QUOTE([create.P(["foo"])])] },
          {
            input: "# foo",
            expected: [create.LIST("O", [create.LIST$item(["foo"])])],
          },
          {
            input: "* foo",
            expected: [create.LIST("U", [create.LIST$item(["foo"])])],
          },
          {
            input: "; foo",
            expected: [create.DL([create.DL$item("T", ["foo"])])],
          },
          {
            input: ": foo",
            expected: [create.DL([create.DL$item("D", ["foo"])])],
          },
        ]);
      });
      describe("多行的块引用", () => {
        theseCasesAreOk([
          ...["> foo\n> bar", "> foo\n> bar\n>"].map((input) => ({
            input,
            expected: [create.QUOTE([create.P(["foo", create.br(), "bar"])])],
          })),
          {
            input: "> foo\n> bar\n> \n> baz",
            expected: [
              create.QUOTE([
                create.P(["foo", create.br(), "bar"]),
                create.P(["baz"]),
              ]),
            ],
          },
        ]);
      });
      describe("多项", () => {
        theseCasesAreOk([
          {
            input: "# foo\n# bar",
            expected: [
              create.LIST("O", [
                create.LIST$item(["foo"]),
                create.LIST$item(["bar"]),
              ]),
            ],
          },
          {
            input: "* foo\n* bar",
            expected: [
              create.LIST("U", [
                create.LIST$item(["foo"]),
                create.LIST$item(["bar"]),
              ]),
            ],
          },
          {
            input: "; foo\n: bar\n: baz",
            expected: [create.DL([
              create.DL$item("T", ["foo"]),
              create.DL$item("D", ["bar"]),
              create.DL$item("D", ["baz"]),
            ])],
          },
        ]);
      });
      describe("延长的项", () => {
        describe("单项", () => {
          theseCasesAreOk([
            {
              input: "* foo\n> bar",
              expected: [create.LIST("U", [
                create.LIST$item(["foo", create.P(["bar"])]),
              ])],
            },
            {
              input: "* foo\n> bar\n> baz",
              expected: [create.LIST("U", [create.LIST$item([
                "foo",
                create.P(["bar", create.br(), "baz"]),
              ])])],
            },
            {
              input: "* foo\n> bar\n>\n> baz",
              expected: [create.LIST("U", [create.LIST$item([
                "foo",
                create.P(["bar"]),
                create.P(["baz"]),
              ])])],
            },
            {
              input: "*\n> bar",
              expected: [create.LIST("U", [create.LIST$item([
                create.P(["bar"]),
              ])])],
            },
          ]);
        });
        describe("多项", () => {
          theseCasesAreOk([
            {
              input: "# foo\n> bar\n# baz",
              expected: [create.LIST("O", [
                create.LIST$item(["foo", create.P(["bar"])]),
                create.LIST$item(["baz"]),
              ])],
            },
            {
              input: "# foo\n>\n# baz",
              expected: [create.LIST("O", [
                create.LIST$item(["foo"]),
                create.LIST$item(["baz"]),
              ])],
            },
            {
              input: ";\n> foo\n> bar\n:\n> baz\n: qux",
              expected: [create.DL([
                create.DL$item("T", [create.P(["foo", create.br(), "bar"])]),
                create.DL$item("D", [create.P(["baz"])]),
                create.DL$item("D", ["qux"]),
              ])],
            },
          ]);
        });
      });
      describe("上下相邻的组的切割", () => {
        theseCasesAreOk([
          {
            input: "* foo\n# bar",
            expected: [
              create.LIST("U", [create.LIST$item(["foo"])]),
              create.LIST("O", [create.LIST$item(["bar"])]),
            ],
          },
          {
            input: "* foo\n> bar\n# baz",
            expected: [
              create.LIST("U", [create.LIST$item(["foo", create.P(["bar"])])]),
              create.LIST("O", [create.LIST$item(["baz"])]),
            ],
          },
          ...["> foo\n# bar", "> foo\n>\n# bar"].map((input) => ({
            input,
            expected: [
              create.QUOTE([create.P(["foo"])]),
              create.LIST("O", [create.LIST$item(["bar"])]),
            ],
          })),
        ]);
      });
      describe("多层深的单项", () => {
        theseCasesAreOk([
          {
            input: "* * foo",
            expected: [
              create.LIST("U", [create.LIST$item([
                create.LIST("U", [create.LIST$item(["foo"])]),
              ])]),
            ],
          },
          {
            input: "* * * foo",
            expected: [
              create.LIST("U", [create.LIST$item([
                create.LIST("U", [create.LIST$item([
                  create.LIST("U", [create.LIST$item(["foo"])]),
                ])]),
              ])]),
            ],
          },
          {
            input: "> * foo",
            expected: [
              create.QUOTE([
                create.LIST("U", [create.LIST$item(["foo"])]),
              ]),
            ],
          },
        ]);
      });
      describe("块引用内", () => {
        theseCasesAreOk([
          { // 块引用套块引用
            input: "> > foo\n> > bar",
            expected: [
              create.QUOTE([
                create.QUOTE([create.P(["foo", create.br(), "bar"])]),
              ]),
            ],
          },
          { // 块引用套其他多项
            input: "> * foo\n> * bar",
            expected: [
              create.QUOTE([create.LIST("U", [
                create.LIST$item(["foo"]),
                create.LIST$item(["bar"]),
              ])]),
            ],
          },
          { // 块引用套有延长的单项·1
            input: "> * foo\n> > bar",
            expected: [
              create.QUOTE([create.LIST("U", [
                create.LIST$item(["foo", create.P(["bar"])]),
              ])]),
            ],
          },
          { // 块引用套有延长的单项·2
            input: "> *\n> > foo\n> > bar\n> >\n> > baz",
            expected: [
              create.QUOTE([create.LIST("U", [
                create.LIST$item([
                  create.P(["foo", create.br(), "bar"]),
                  create.P(["baz"]),
                ]),
              ])]),
            ],
          },
        ]);
      });
      describe("延长的项内", () => {
        theseCasesAreOk([
          ...["# * foo\n> * bar", "#\n> * foo\n> * bar"].map((input) => ({
            input, // 延长项套多项
            expected: [create.LIST("O", [create.LIST$item([
              create.LIST("U", [
                create.LIST$item(["foo"]),
                create.LIST$item(["bar"]),
              ]),
            ])])],
          })),
          ...["# * foo\n>\n> * bar", "#\n> * foo\n>\n> * bar"].map((input) => ({
            input, // 延长项套两组由空行隔开的单项
            expected: [create.LIST("O", [create.LIST$item([
              create.LIST("U", [create.LIST$item(["foo"])]),
              create.LIST("U", [create.LIST$item(["bar"])]),
            ])])],
          })),
          { // 延长项套延长项·1
            input: "# # foo\n> > bar",
            expected: [create.LIST("O", [create.LIST$item([
              create.LIST("O", [create.LIST$item(["foo", create.P(["bar"])])]),
            ])])],
          },
          { // 延长项套延长项·2
            input: "#\n> #\n> > foo\n> >\n> > bar",
            expected: [create.LIST("O", [create.LIST$item([
              create.LIST("O", [create.LIST$item(
                [create.P(["foo"]), create.P(["bar"])],
              )]),
            ])])],
          },
        ]);
      });
      describe("组的切割导致的内层的切割", () => {
        theseCasesAreOk([
          { // 内层看似同组
            input: "# * foo\n* * bar",
            expected: [
              create.LIST("O", [create.LIST$item(
                [create.LIST("U", [create.LIST$item(["foo"])])],
              )]),
              create.LIST("U", [create.LIST$item(
                [create.LIST("U", [create.LIST$item(["bar"])])],
              )]),
            ],
          },
          { // 内层看似延长
            input: "# * foo\n* > bar",
            expected: [
              create.LIST("O", [create.LIST$item(
                [create.LIST("U", [create.LIST$item(["foo"])])],
              )]),
              create.LIST("U", [create.LIST$item(
                [create.QUOTE([create.P(["bar"])])],
              )]),
            ],
          },
        ]);
      });
      describe("多层参差不齐", () => {
        theseCasesAreOk([
          {
            input: "# foo\n> # bar",
            expected: [create.LIST("O", [create.LIST$item([
              "foo",
              create.LIST("O", [create.LIST$item(["bar"])]),
            ])])],
          },
          {
            input: "#\n> foo\n> # bar",
            expected: [create.LIST("O", [create.LIST$item([
              create.P(["foo"]),
              create.LIST("O", [create.LIST$item(["bar"])]),
            ])])],
          },
          ...["# # foo\n> bar", "#\n> # foo\n> bar"].map((input) => ({
            input,
            expected: [create.LIST("O", [create.LIST$item([
              create.LIST("O", [create.LIST$item(["foo"])]),
              create.P(["bar"]),
            ])])],
          })),
          {
            input: "# #\n> > foo\n> bar",
            expected: [create.LIST("O", [create.LIST$item([
              create.LIST("O", [create.LIST$item([create.P(["foo"])])]),
              create.P(["bar"]),
            ])])],
          },
        ]);
      });
    });
    describe("表格", () => {
      describe("能正确解析 “预期产物是单行” 的、“源代码使用 ‘单行格式’” 的表格", () => {
        theseCasesAreOk([
          {
            input: "{|\n| a\n|}",
            expected: [create.TABLE(null, [[create.TABLE$cell("D", ["a"])]])],
          },
          {
            input: "{|\n| a || b\n|}",
            expected: [create.TABLE(null, [
              [create.TABLE$cell("D", ["a"]), create.TABLE$cell("D", ["b"])],
            ])],
          },
          {
            input: "{|\n! a !! b\n|}",
            expected: [create.TABLE(null, [
              [create.TABLE$cell("H", ["a"]), create.TABLE$cell("H", ["b"])],
            ])],
          },
          {
            input: "{|\n! a || b\n|}",
            expected: [create.TABLE(null, [
              [create.TABLE$cell("H", ["a"]), create.TABLE$cell("D", ["b"])],
            ])],
          },
          // { // FIXME?: 空白被保留了下来，要保留吗？
          //   input: "{|\n| a || b \n|}",
          //   expected: [create.TABLE(null, [
          //     [create.TABLE$cell("D", ["a"]), create.TABLE$cell("D", ["b"])],
          //   ])],
          // },
          {
            input: "{|\n| [`foo`] || ['bar']\n|}",
            expected: [create.TABLE(null, [
              [
                create.TABLE$cell("D", [create.code("foo")]),
                create.TABLE$cell("D", [create.em("strong", ["bar"])]),
              ],
            ])],
          },
        ]);
      });
      describe("能正确解析 “预期产物是单行” 的、“源代码使用 ‘多行格式’” 的表格", () => {
        theseCasesAreOk([
          {
            input: "{|\n| a\n| b\n|}",
            expected: [create.TABLE(null, [
              [create.TABLE$cell("D", ["a"]), create.TABLE$cell("D", ["b"])],
            ])],
          },
        ]);
      });
      describe("能正确解析 “预期产物是单行” 的、“源代码使用 ‘两种格式混合’” 的表格", () => {
        theseCasesAreOk([
          {
            input: "{|\n| a || b\n| c || d\n| e\n|}",
            expected: [create.TABLE(null, [
              [
                create.TABLE$cell("D", ["a"]),
                create.TABLE$cell("D", ["b"]),
                create.TABLE$cell("D", ["c"]),
                create.TABLE$cell("D", ["d"]),
                create.TABLE$cell("D", ["e"]),
              ],
            ])],
          },
        ]);
      });
      describe("能正确解析 “预期产物是多行” 的表格", () => {
        theseCasesAreOk([
          {
            input: "{|\n! a !! b\n|-\n| c || d\n|-\n| e || f\n|}",
            expected: [create.TABLE(null, [
              [create.TABLE$cell("H", ["a"]), create.TABLE$cell("H", ["b"])],
              [create.TABLE$cell("D", ["c"]), create.TABLE$cell("D", ["d"])],
              [create.TABLE$cell("D", ["e"]), create.TABLE$cell("D", ["f"])],
            ])],
          },
        ]);
      });
      describe("能正确解析源代码中表格的 “行内最后的单元的延续”", () => {
        theseCasesAreOk([
          { // NOTE: MediaWiki 也是如此：将首行与其余行的内容分开为行内元素与块元素
            input: "{|\n| foo\nbar\n|}",
            expected: [
              create.TABLE(null, [[
                create.TABLE$cell("D", ["foo", create.P(["bar"])]),
              ]]),
            ],
          },
          {
            input: "{|\n|\nfoo\n|}",
            expected: [
              create.TABLE(null, [[
                create.TABLE$cell("D", [create.P(["foo"])]),
              ]]),
            ],
          },
          {
            input: "{|\n| foo ||\nbar\n|}",
            expected: [
              create.TABLE(null, [[
                create.TABLE$cell("D", ["foo"]),
                create.TABLE$cell("D", [create.P(["bar"])]),
              ]]),
            ],
          },
          {
            input: "{|\n| foo\n\n= bar =\n|}",
            expected: [
              create.TABLE(null, [[
                create.TABLE$cell("D", ["foo", create.H(1, ["bar"])]),
              ]]),
            ],
          },
          {
            input: "{|\n|\n== foo ==\n|}",
            expected: [
              create.TABLE(null, [[
                create.TABLE$cell("D", [create.H(2, ["foo"])]),
              ]]),
            ],
          },
          ...[
            "{|\n|\n== foo ==\n== bar ==\n|}",
            "{|\n|\n== foo ==\n\n== bar ==\n|}",
          ].map((input) => ({
            input,
            expected: [
              create.TABLE(null, [[
                create.TABLE$cell("D", [
                  create.H(2, ["foo"]),
                  create.H(2, ["bar"]),
                ]),
              ]]),
            ],
          })),
          {
            input: "{|\n|\n* foo\n\n|}",
            expected: [
              create.TABLE(null, [[
                create.TABLE$cell("D", [
                  create.LIST("U", [create.LIST$item(["foo"])]),
                ]),
              ]]),
            ],
          },
          {
            input: "{|\n|\n* foo\n> bar\n|}",
            expected: [
              create.TABLE(null, [[
                create.TABLE$cell("D", [
                  create.LIST("U", [
                    create.LIST$item(["foo", create.P(["bar"])]),
                  ]),
                ]),
              ]]),
            ],
          },
        ]);
      });
      describe("能正确解析标题（caption）", () => {
        theseCasesAreOk([
          {
            input: "{|\n|+ foo\n|-\n| bar\n|}",
            expected: [
              create.TABLE(["foo"], [[create.TABLE$cell("D", ["bar"])]]),
            ],
          },
        ]);
      });
      describe("regressions", () => {
        theseCasesAreOk([
          {
            input: `
              {|
              |+ 文本相关样式一览
              |-
              ! 功能 !! 形式 !! 效果
              |-
              | 加粗 || [\`['这样']\`] || ['这样']
              |-
              | 删除线 || [\`[~这样~]\`] || [~这样~]
              |}`.trim().split("\n").map((l) => l.trimStart()).join("\n"),
            expected: [
              create.TABLE(["文本相关样式一览"], [
                [
                  create.TABLE$cell("H", ["功能"]),
                  create.TABLE$cell("H", ["形式"]),
                  create.TABLE$cell("H", ["效果"]),
                ],
                [
                  create.TABLE$cell("D", ["加粗"]),
                  create.TABLE$cell("D", [create.code("['这样']")]),
                  create.TABLE$cell("D", [create.em("strong", ["这样"])]),
                ],
                [
                  create.TABLE$cell("D", ["删除线"]),
                  create.TABLE$cell("D", [create.code("[~这样~]")]),
                  create.TABLE$cell("D", [create.s(["这样"])]),
                ],
              ]),
            ],
          },
        ]);
      });
    });
  });
});
