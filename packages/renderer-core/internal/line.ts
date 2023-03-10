import { decodeHTMLStrict } from "entities";

import { intersperse } from "./utils";

type VChildren<VNode> = (string | VNode)[];

function unescape(text: string) {
  return decodeHTMLStrict(text);
}

/**
 * 将由 VNode 和字符串组成的数组中，相邻的字符串合并。
 *
 * 例如：
 * * `["1", h("2"), "3", "4"]` => `["1", h("2"), "34"]`
 *
 * @param inlines
 * @param unescapes 是否反转译文本。（如 HTML Entities）
 * @returns
 */
export function joinInlines<VNode>(
  inlines: VChildren<VNode>,
  unescapes = true,
) {
  const [result, remStr] = inlines.reduce((acc, cur) => {
    const [nodes, str] = acc;
    if (typeof cur === "string") {
      if (!str) return [nodes, cur];
      return [nodes, str + cur];
    }
    if (!str) return [[...nodes, cur], null];
    const decodedStr = unescapes ? unescape(str) : str;
    return [[...nodes, decodedStr, cur], null];
  }, [[] as VChildren<VNode>, null as string | null]);
  if (remStr) {
    const decodedStr = unescapes ? unescape(remStr) : remStr;
    result.push(decodedStr);
  }
  return result;
}

export function joinLines<VNode extends string>(
  lines: VChildren<VNode>[],
  breaks: boolean,
  h: (_1, _2?, _3?) => VNode | string,
) {
  // TODO: 也许可以基于前后相连的字符种类决定是使用空格还是空字符串拼接两行
  lines = intersperse(lines, breaks ? [h("br")] : [" "]);
  return joinInlines(lines.flat(), false);
}
