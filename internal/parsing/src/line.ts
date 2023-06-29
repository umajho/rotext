import { decodeHTMLStrict } from "entities";

import { create, type InlineSlot } from "@rotext/nodes";

import { intersperseWithFactory } from "./utils";

function unescape(text: string) {
  return decodeHTMLStrict(text);
}

/**
 * 将行内相邻的文本节点合并。
 *
 * 例如：
 * * `["1", h("2"), "3", "4"]` => `["1", h("2"), "34"]`
 *
 * @param inlines
 * @param unescapes 是否反转译文本。（如 HTML Entities）
 * @returns
 */
export function joinInlines(
  inlines: InlineSlot,
  unescapes = true,
): InlineSlot {
  const [result, remStr] = inlines.reduce((acc, cur) => {
    const [nodes, str] = acc;
    if (typeof cur === "string") {
      if (!str) return [nodes, cur];
      return [nodes, str + cur];
    }
    if (!str) return [[...nodes, cur], null];
    const decodedStr = unescapes ? unescape(str) : str;
    return [[...nodes, decodedStr, cur], null];
  }, [[] as InlineSlot, null as string | null]);
  if (remStr) {
    const decodedStr = unescapes ? unescape(remStr) : remStr;
    result.push(decodedStr);
  }
  return result;
}

export function joinLines(
  lines: InlineSlot[],
  breaks: boolean,
): InlineSlot {
  // TODO: 也许可以基于前后相连的字符种类决定是使用空格还是空字符串拼接两行
  lines = intersperseWithFactory(
    lines,
    () => breaks ? [create.br()] : [create.text(" ")],
  );
  return joinInlines(lines.flat(), false);
}
