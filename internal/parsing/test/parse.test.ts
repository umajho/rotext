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
    expect(output).toStrictEqual(create.ROOT(expected));
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
          { input: "foo", expected: [create.P([create.text("foo")])] },
        ]);
      });
      describe("转义（基础）", () => {
        // NOTE: 这里没有涉及的元素的测试由测试对应元素的地方负责

        describe("于原先的单行块元素开头", () => {
          theseCasesAreOk([
            {
              input: String.raw`\---`,
              expected: [create.P([create.text("---")])],
            },
          ]);
        });
        describe("于行内（基础：对自身的转义及对无特殊意义字符的转义）", () => {
          theseCasesAreOk([
            {
              input: String.raw`\\`,
              expected: [create.P([create.text("\\")])],
            },
            {
              input: String.raw`\a`,
              expected: [create.P([create.text("a")])],
            },
            {
              input: "\\",
              expected: [create.P([create.text("\\")])],
            },
            {
              input: "a\\",
              expected: [create.P([create.text("a\\")])],
            },
          ]);
        });
      });
      describe("多行", () => {
        it("`breaks` 选项为假时，输入文本中的单次换行只视为空格", () => {
          assertOk(
            "foo\nbar",
            [create.P([create.text("foo bar")])],
            { breaks: false },
          );
        });
        it("`breaks` 选项为真时，输入文本中的单次换行视为换行", () => {
          assertOk(
            "foo\nbar",
            [create.P([create.text("foo"), create.br(), create.text("bar")])],
            { breaks: true },
          );
        });
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
            expected: [create.P([create.code("[`foo"), create.text("`]")])],
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
            expected: [create.P([create.em("strong", [create.text("foo")])])],
          })),
          ...["[/foo/]", "//foo//"].map((input) => ({
            input,
            expected: [create.P([create.em(null, [create.text("foo")])])],
          })),
          ...["[_foo_]", "__foo__"].map((input) => ({
            input,
            expected: [create.P([create.u([create.text("foo")])])],
          })),
          ...["[~foo~]", "~~foo~~"].map((input) => ({
            input,
            expected: [create.P([create.s([create.text("foo")])])],
          })),
          ...["[.foo.]"].map((input) => ({
            input,
            expected: [create.P([create.em("dotted", [create.text("foo")])])],
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
          { input: "[]", expected: [create.P([create.text("[]")])] },
          { input: "[foo]", expected: [create.P([create.text("[foo]")])] },
          { input: "['foo", expected: [create.P([create.text("['foo")])] },
        ]);
      });
      describe("转义", () => {
        theseCasesAreOk([
          ...["\\['foo']", "[\\'foo']", "['foo\\']", "['foo'\\]"].map(
            (input) => ({
              input,
              expected: [create.P([create.text("['foo']")])],
            }),
          ),
          {
            input: String.raw`['\foo']`,
            expected: [
              create.P([create.em("strong", [create.text("foo")])]),
            ],
          },
          {
            input: String.raw`['\['foo']`,
            expected: [
              create.P([create.em("strong", [create.text("['foo")])]),
            ],
          },
          {
            input: String.raw`['foo\']']`,
            expected: [
              create.P([create.em("strong", [create.text("foo']")])]),
            ],
          },
        ]);
      });
    });
  });
  describe("块级元素", () => {
    describe("段落", () => {
      it("两次换行开启新的段落", () => {
        assertOk("foo\n\nbar", [
          create.P([create.text("foo")]),
          create.P([create.text("bar")]),
        ]);
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
            expected: [create.H(n as any, [create.text("foo")])],
          });
        }

        theseCasesAreOk([
          ...boringCases,
          {
            input: "foo\n= bar =\nbaz",
            expected: [
              create.P([create.text("foo")]),
              create.H(1, [create.text("bar")]),
              create.P([create.text("baz")]),
            ],
          },
        ]);
      });
      describe("多余的标记符号会留下来", () => {
        theseCasesAreOk([
          {
            input: "== foo =",
            expected: [create.H(1, [create.text("= foo")])],
          },
          {
            input: "= foo ==",
            expected: [create.H(1, [create.text("foo =")])],
          },
          {
            input: "======= foo =======",
            expected: [create.H(6, [create.text("= foo =")])],
          },
        ]);
      });
      describe("内部可以有行内元素", () => {
        theseCasesAreOk([
          {
            input: "== ['foo'] ==",
            expected: [
              create.H(2, [create.em("strong", [create.text("foo")])]),
            ],
          },
        ]);
      });
    });
  });
});
