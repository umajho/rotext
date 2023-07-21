import { decodeHTMLStrict } from "entities";

import { create, type InlineSlot, MixedSlot } from "@rotext/nodes";

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

/**
 * 将多行合并为一行。
 *
 * @param lines 要合并的各行，在传入前应已经使用 `joinInlines` 整理好。传入后不能再使用
 * @param breaks 是否视「软换行」为换行
 * @returns 合并后的一行
 *
 * TODO: 也许可以基于前后相连的字符种类决定是使用空格还是空字符串拼接两行
 */
export function joinLines(
  lines: InlineSlot[],
  breaks: boolean,
): InlineSlot {
  if (!lines.length) return [];

  const result: InlineSlot = lines[0];
  for (let i = 1; i < lines.length; i++) {
    const line = lines[i];
    appendLine(result, line, breaks);
  }

  return result;
}

function appendLine(
  target: InlineSlot,
  line: InlineSlot,
  breaks: boolean,
) {
  const topEnd = target[target.length - 1];
  const bottomStart = line[0];

  if (
    // NOTE: 规定行内元素必定小写字母打头，以此来判断
    (typeof topEnd === "string" || startsWithLowerCase(topEnd.type)) &&
    (typeof bottomStart === "string" || startsWithLowerCase(bottomStart.type))
  ) {
    if (breaks) {
      target.push(create.br(), ...line);
    } else {
      target[target.length - 1] = topEnd + " " + bottomStart;
      target.push(...line.slice(1));
    }
  } else {
    target.push(...line);
  }
}

function startsWithLowerCase(x: string) {
  return x[0].toLowerCase() === x[0];
}

/**
 * `opts`:
 * - `paragraph`:
 *   - `"default"`: 如果先前有段落，将 line 追加入段落，否则创建新的段落放入 line。
 *   - `"new"`: 无论如何都创建新的段落放入 line。
 *   - `"no"`: line 与段落平级铺在外侧。
 */
export function appendLineToMixedSlot(
  slot: MixedSlot,
  line: InlineSlot,
  opts: {
    paragraph: "default" | "new" | "no";
    breaks: boolean;
    recording?: <T>(el: T) => T;
  },
) {
  if (opts.paragraph === "no") {
    slot.push(...line);
  } else if (opts.paragraph === "new") {
    const p = create.P(line);
    slot.push(opts.recording ? opts.recording(p) : p);
  } else {
    const last = slot[slot.length - 1];
    if (last && typeof last !== "string" && last.type === "P") {
      appendLine(last.slot, line, opts.breaks);
    } else {
      const p = create.P(line);
      slot.push(opts.recording ? opts.recording(p) : p);
    }
  }
}

/**
 * 就地去掉一串行内元素开头与结尾的空白。不深入。
 */
export function trimInlinesEndShallowInPlace(
  inlines: InlineSlot,
): [result: InlineSlot, changed: boolean] {
  let last = inlines[inlines.length - 1];
  if (typeof last !== "string") return [inlines, false];

  const oldLength = last.length;
  last = last.trimEnd();
  if (oldLength === last.length) return [inlines, false];

  if (last.length) {
    inlines[inlines.length - 1] = last;
  } else {
    inlines = inlines.slice(0, inlines.length - 1);
  }
  return [inlines, true];
}
