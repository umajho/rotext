import { describe, expect, it } from "vitest";

import * as nodes from "@rotext/nodes";
import type { RootElement } from "@rotext/nodes";

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
    expect(output).toStrictEqual(nodes.root(expected));
  }
}

function theseCasesAreOk(cases: Case[], opts?: ParseOptions) {
  for (const [i, theCase] of cases.entries()) {
    it(`case ${i + 1}: \`${theCase.input}\` ok`, () => {
      expect(parse(theCase.input, opts))
        .toStrictEqual(nodes.root(theCase.expected));
    });
  }
}

describe("解析", () => {
  describe("行内元素", () => {
    describe("文本", () => {
      describe("一般内容", () => {
        theseCasesAreOk([
          {
            input: "foo",
            expected: [nodes.blockParagraph([nodes.text("foo")])],
          },
        ]);
      });
      describe("转义", () => {
        theseCasesAreOk([
          {
            input: String.raw`\---`,
            expected: [nodes.blockParagraph([nodes.text("---")])],
          },
          {
            input: String.raw`\\`,
            expected: [nodes.blockParagraph([nodes.text("\\")])],
          },
          {
            input: String.raw`\a`,
            expected: [nodes.blockParagraph([nodes.text("a")])],
          },
          {
            input: "\\",
            expected: [nodes.blockParagraph([nodes.text("\\")])],
          },
          {
            input: "a\\",
            expected: [nodes.blockParagraph([nodes.text("a\\")])],
          },
        ]);
      });
      describe("多行", () => {
        it("`breaks` 选项为假时，输入文本中的单次换行只视为空格", () => {
          assertOk(
            "foo\nbar",
            [nodes.blockParagraph([nodes.text("foo bar")])],
            { breaks: false },
          );
        });
        it("`breaks` 选项为真时，输入文本中的单次换行视为换行", () => {
          assertOk(
            "foo\nbar",
            [nodes.blockParagraph([
              nodes.text("foo"),
              nodes.inlineBreak(),
              nodes.text("bar"),
            ])],
            { breaks: true },
          );
        });
      });
    });
  });
  describe("块级元素", () => {
    describe("段落", () => {
      it("两次换行开启新的段落", () => {
        assertOk("foo\n\nbar", [
          nodes.blockParagraph([nodes.text("foo")]),
          nodes.blockParagraph([nodes.text("bar")]),
        ]);
      });
    });
    describe("Thematic Break", () => {
      describe("能正确解析", () => {
        theseCasesAreOk([
          { input: "---", expected: [nodes.blockThematicBreak()] },
        ]);
      });
    });
  });
});
