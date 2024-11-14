//! 在完成新解析器之前的临时方案，会在新解析器完成之后移除。

import {
  Marked,
  type RendererObject,
  type TokenizerAndRendererExtension,
} from "marked"; // TODO!!: 最终去除此依赖。
import { h } from "snabbdom";
import toHTML from "snabbdom-to-html";
import { HtmlValidate } from "html-validate";

const extInternalLink: TokenizerAndRendererExtension = {
  name: "internalLink",
  level: "inline",
  start: (src) => src.match(/\[\[.+?(?:\|.+?)?\]\]/)?.index,
  tokenizer: (src, _tokens) => {
    const rule = /^\[\[(.+?)(?:\|(.+?))?\]\]/;
    const match = rule.exec(src);
    if (!match) return;
    return {
      type: "internalLink",
      raw: match[0],
      tokens: [],

      displayName: match[2] ?? match[1],
      ...(match[2] ? { pageName: match[1] } : {}),
    };
  },
  renderer: (token) => {
    return toHTML(
      h(
        "x-internal-link",
        { attrs: { "address": token.pageName ?? token.displayName } },
        h("span", { attrs: { slot: "content" } }, token.displayName),
      ),
    );
  },
};

const allowedArguments = new Set(["use-fixtures"]);
const allowedContents = new Set(["input", "expected", "empty"]);
const allowedArgumentsForFixture = new Set(["name"]);
const allowedContentFixture = new Set(["input"]);

const renderer: RendererObject = {
  heading({ tokens, depth }) {
    const text = this.parser.parseInline(tokens);
    return toHTML(
      h(`h${depth}`, {
        props: { innerHTML: text },
        attrs: { id: text },
      }),
    );
  },

  code({ lang, text }) {
    if (!lang || !/^example(-fixture)?($|\s)/.test(lang)) return false;

    const isFixture = lang.startsWith("example-fixture");

    const args = parseExampleLangArguments(lang);
    const content = parseExampleContent(text);

    if (isFixture) {
      assertKeysIn(args, {
        set: allowedArgumentsForFixture,
        what: "example fixture argument",
      });
      assertKeysIn(content, {
        set: allowedContentFixture,
        what: "example fixture content",
      });
    } else {
      assertKeysIn(args, { set: allowedArguments, what: "example argument" });
      assertKeysIn(content, { set: allowedContents, what: "example content" });
      if ("expected" in content) {
        const contentExpected = content["expected"] as string;
        assertHTMLValid(contentExpected);
        content["expected"] = contentExpected;
      }
    }

    if (!isFixture) {
      if (content["empty"] && content["expected"]) {
        throw new Error("stated that expected output is empty, but it is not");
      } else if (!content["empty"] && !content["expected"]) {
        throw new Error(
          "didn't state that expected output is empty, but it is",
        );
      }
    }

    return toHTML(
      h(isFixture ? "x-rotext-example-fixture" : "x-rotext-example", {
        attrs: {
          ...args,
          ...content,
        },
      }),
    );
  },
};

function assertKeysIn(input: { [key: string]: any }, opts: {
  set: Set<string>;
  what: string;
}) {
  for (const key of Object.keys(input)) {
    if (!opts.set.has(key)) {
      throw new Error(
        `${opts.what} key “${key}” not in ${JSON.stringify([...opts.set])}`,
      );
    }
  }
}

function assertHTMLValid(input: string) {
  const v = new HtmlValidate({
    rules: {
      "prefer-tbody": "off",
    },
  });
  const report = v.validateStringSync(input);

  const results = report.results;
  // .filter((r) => {
  //   r.messages = r.messages.filter((m) => {
  //     // …
  //     return true;
  //   });
  //   return !!r.messages.length;
  // });
  if (!results.length) return;

  throw new Error(`invalid HTML: ${JSON.stringify(report.results)}`);
}

function parseExampleLangArguments(lang: string) {
  const args = lang.split(/\s+/).filter((arg) => !!arg.trim()).slice(1);
  const argPairs = args.map(
    (arg) => /(.+?)=(.+)/.exec(arg)!.slice(1, 3) as [string, string],
  );

  const argMap: { [key: string]: string } = {};
  for (const [key, value] of argPairs) {
    if (key in argMap) {
      throw new Error(`duplicate example lang argument key “${key}”`);
    }
    argMap[key] = value;
  }

  return argMap;
}

function parseExampleContent(raw: string) {
  const content: { [key: string]: string | true } = {};

  const parts = raw.split(/(?:\n|^)·{3,}/);

  const [input] = parts.splice(0, 1) as [string];
  if (input.trim()) {
    content["input"] = input;
  }

  const partPairs = parts.map((part): [string, string | true] => {
    if (part.indexOf("\n") < 0) return [part, true];

    const g = /^(.*?)\n((?:.|\n)*)$/.exec(part);
    return g!.slice(1, 3) as [string, string];
  });

  for (let [key, value] of partPairs) {
    key ||= "expected";
    if (key in content) {
      throw new Error(`duplicate example content key “${key}”`);
    }
    content[key] = value;
  }

  return content;
}

const marked = new Marked({
  extensions: [extInternalLink],
  useNewRenderer: true,
  renderer,
});

export function parseMarkdown(input: string): string {
  input = input.replace(/<wbr \/>\n\s*?/g, "&#x200B;");

  let parsed = marked.parse(input) as string;
  parsed = parsed.replace(/&#x200B;/g, "");

  return parsed;
}
