import { describe, expect, it } from "vitest";

import { create, type RootElement } from "@rotext/nodes";

import * as parser from "../src/rotext";

interface ParseOptions {
  breaks?: boolean;
}

function parse(
  input: string,
  opts: ParseOptions = { breaks: true },
): RootElement {
  return parser.parse(input, opts) as RootElement;
}

interface Case {
  input: string;
  expected: RootElement["slot"];
}

function assertOk(
  input: string,
  expected: RootElement["slot"] | null,
  opts?: ParseOptions,
) {
  const output = parse(input, opts);
  if (expected) {
    expect(create.ROOT(expected)).toStrictEqual(output);
  }
}

function theseCasesAreOk(cases: Case[], opts?: ParseOptions) {
  for (const [i, theCase] of cases.entries()) {
    it(`case ${i + 1}: \`${theCase.input}\` ok`, () => {
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
        it("`breaks` 选项为假时，输入文本中的单次换行只视为空格", () => {
          assertOk(
            "foo\nbar",
            [create.P(["foo bar"])],
            { breaks: false },
          );
        });
        it("`breaks` 选项为真时，输入文本中的单次换行视为换行", () => {
          assertOk(
            "foo\nbar",
            [create.P(["foo", create.br(), "bar"])],
            { breaks: true },
          );
        });
      });
    });
    describe("引用链接", () => {
      describe("能正确解析格式正确的", () => {
        theseCasesAreOk(
          [
            "TP.42",
            ...["TP.abc", "abc", "~"],
            ...["TP.abc#123", "abc#123", "#123", "#"],
            ...["TP.abc/def", "abc/def", "~/def"],
            ...["TP.abc/def#456", "abc/def#456", "~/def#456"],
          ]
            .map((input) => ({
              input: `>>${input}`,
              expected: [create.P([create.ref_link(input)])],
            })),
        );
      });
      describe("忽略格式不正确的", () => {
        theseCasesAreOk(
          [
            ...[
              ...["", "42"],
              ...["TP.~", "TP.#123", "TP.#"],
              ...["/def", "TP.~/def"],
            ].map((input) => ({
              input: `>>${input}`,
              expected: [create.P([`>>${input}`])],
            })),

            ...[
              { parsed: "#123", remain: "/def" }, // `>>#123/456`
              { parsed: "TP.abc#123", remain: "/def" }, // `>>TP.abc#123/def`
            ].map(({ parsed, remain }) => ({
              input: `>>${parsed}${remain}`,
              expected: [
                create.P([create.ref_link(parsed), remain]),
              ],
            })),
          ],
        );
      });
      describe("可以位于行首", () => {
        theseCasesAreOk([
          {
            input: ">>TP.42",
            expected: [create.P([create.ref_link("TP.42")])],
          },
        ]);
      });
    });
    describe("行内代码文本", () => {
      describe("能正确解析", () => {
        theseCasesAreOk([
          ...["[`foo`]", "``foo``"].map((input) => ({
            input,
            expected: [create.P([create.code("foo")])],
          })),
          {
            input: "[`['foo']`]",
            expected: [create.P([create.code("['foo']")])],
          },
        ]);
      });
      describe("贪婪", () => {
        theseCasesAreOk([
          {
            input: "[`[`foo`]`]",
            expected: [create.P([create.code("[`foo"), "`]"])],
          },
        ]);
      });
      describe("目前即使内容为空也会创建", () => {
        theseCasesAreOk([
          { input: "[``]", expected: [create.P([create.code("")])] },
        ]);
      });
      describe("不存在转义", () => {
        theseCasesAreOk([
          { input: "[`\\a`]", expected: [create.P([create.code("\\a")])] },
          { input: "[`\\`]", expected: [create.P([create.code("\\")])] },
        ]);
      });
    });
    describe("文本相关样式", () => {
      describe("能正确解析", () => {
        theseCasesAreOk([
          ...["['foo']", "''foo''"].map((input) => ({
            input,
            expected: [create.P([create.em("strong", ["foo"])])],
          })),
          ...["[/foo/]", "//foo//"].map((input) => ({
            input,
            expected: [create.P([create.em(null, ["foo"])])],
          })),
          ...["[_foo_]", "__foo__"].map((input) => ({
            input,
            expected: [create.P([create.u(["foo"])])],
          })),
          ...["[~foo~]", "~~foo~~"].map((input) => ({
            input,
            expected: [create.P([create.s(["foo"])])],
          })),
          ...["[.foo.]"].map((input) => ({
            input,
            expected: [create.P([create.em("dotted", ["foo"])])],
          })),
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
            input: "['~~foo~~']",
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
    describe("旁注", () => {
      describe("能正确解析", () => {
        theseCasesAreOk([
          {
            input: "[foo(bar)]",
            expected: [
              create.P([create.ruby(["foo"], ["(", ")"], ["bar"])]),
            ],
          },
          {
            input: "[测(•)][试(•)]",
            expected: [
              create.P([
                create.ruby(["测"], ["(", ")"], ["•"]),
                create.ruby(["试"], ["(", ")"], ["•"]),
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
                create.ruby(["测试"], ["（", "）"], ["test"]),
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
                create.ruby([create.code("foo")], ["(", ")"], ["bar"]),
              ]),
            ],
          },
          {
            input: "[foo([`bar`])]",
            expected: [
              create.P([
                create.ruby(["foo"], ["(", ")"], [create.code("bar")]),
              ]),
            ],
          },
          {
            input: "[1 [`2`] 3(4 [5(6)] 7)]",
            expected: [
              create.P([
                create.ruby(
                  ["1 ", create.code("2"), " 3"],
                  ["(", ")"],
                  ["4 ", create.ruby(["5"], ["(", ")"], ["6"]), " 7"],
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
            input: "foo\n= bar =\nbaz",
            expected: [
              create.P(["foo"]),
              create.H(1, ["bar"]),
              create.P(["baz"]),
            ],
          },
        ]);
      });
      describe("多余的标记符号会留下来", () => {
        theseCasesAreOk([
          { input: "== foo =", expected: [create.H(1, ["= foo"])] },
          { input: "= foo ==", expected: [create.H(1, ["foo ="])] },
          {
            input: "======= foo =======",
            expected: [create.H(6, ["= foo ="])],
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
    describe("表格", () => {
      describe("能正确解析 “预期产物是单行” 的、“源代码使用 ‘单行格式’” 的表格", () => {
        theseCasesAreOk([
          {
            input: "{|\n|a\n|}",
            expected: [create.TABLE(null, [[create.TABLE$cell("D", ["a"])]])],
          },
          {
            input: "{|\n|a||b\n|}",
            expected: [create.TABLE(null, [
              [create.TABLE$cell("D", ["a"]), create.TABLE$cell("D", ["b"])],
            ])],
          },
          {
            input: "{|\n!a!!b\n|}",
            expected: [create.TABLE(null, [
              [create.TABLE$cell("H", ["a"]), create.TABLE$cell("H", ["b"])],
            ])],
          },
          {
            input: "{|\n!a||b\n|}",
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
            input: "{|\n|[`foo`]||['bar']\n|}",
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
            input: "{|\n|a\n|b\n|}",
            expected: [create.TABLE(null, [
              [create.TABLE$cell("D", ["a"]), create.TABLE$cell("D", ["b"])],
            ])],
          },
        ]);
      });
      describe("能正确解析 “预期产物是单行” 的、“源代码使用 ‘两种格式混合’” 的表格", () => {
        theseCasesAreOk([
          {
            input: "{|\n|a||b\n|c||d\n|e\n|}",
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
            input: "{|\n!a!!b\n|-\n|c||d\n|-\n|e||f\n|}",
            expected: [create.TABLE(null, [
              [create.TABLE$cell("H", ["a"]), create.TABLE$cell("H", ["b"])],
              [create.TABLE$cell("D", ["c"]), create.TABLE$cell("D", ["d"])],
              [create.TABLE$cell("D", ["e"]), create.TABLE$cell("D", ["f"])],
            ])],
          },
        ]);
      });
      describe("能正确解析源代码中 “单个单元格有多行” 的表格", () => {
        theseCasesAreOk([
          { // NOTE: MediaWiki 也是如此处理的
            input: "{|\n|foo\nbar\n|}",
            expected: [
              create.TABLE(null, [[
                create.TABLE$cell("D", ["foo", create.P(["bar"])]),
              ]]),
            ],
          },
          { // 同上
            input: "{|\n|\nfoo\n|}",
            expected: [
              create.TABLE(null, [[
                create.TABLE$cell("D", [create.P(["foo"])]),
              ]]),
            ],
          },
          { // 同上
            input: "{|\n|foo\n\n=bar=\n|}",
            expected: [
              create.TABLE(null, [[
                create.TABLE$cell("D", ["foo", create.H(1, ["bar"])]),
              ]]),
            ],
          },
        ]);
      });
      describe("能正确解析标题（caption）", () => {
        theseCasesAreOk([
          {
            input: "{|\n|+foo\n|-\n|bar\n|}",
            expected: [
              create.TABLE(["foo"], [[create.TABLE$cell("D", ["bar"])]]),
            ],
          },
        ]);
      });
    });
  });
});
