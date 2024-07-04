//! 在完成新解析器之前的临时方案，会在新解析器完成之后移除。

import {
  Marked,
  type RendererObject,
  type TokenizerAndRendererExtension,
} from "marked"; // TODO!!: 最终去除此依赖。
import { h } from "snabbdom";
import toHTML from "snabbdom-to-html"; // TODO!!: 最终去除此依赖。

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
        token.pageName ? { attrs: { "page-name": token.pageName } } : {},
        token.displayName,
      ),
    );
  },
};

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

    const args = parseExampleLangArguments(lang);
    const content = parseExampleContent(text);

    {
      const argSet = new Set(Object.keys(args));
      if (Object.keys(content).some((c) => argSet.has(c))) {
        throw new Error(`lang arguments and content cannot have common key`);
      }
    }

    return toHTML(
      h("x-rotext-example", {
        attrs: {
          ...args,
          ...content,
        },
      }),
    );
  },
};

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

  for (const [key_, value] of partPairs) {
    const key = key_ || "expected";
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
