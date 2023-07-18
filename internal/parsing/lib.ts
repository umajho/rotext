import { Document } from "@rotext/nodes";

import * as parser from "./src/rotext";

export interface ParseOptions {
  /**
   * 将软换行视为 br 还是空格。
   */
  softBreakAs?: "br" | "space";

  /**
   * 是否记录位置信息。（目前只对部分块元素的进行记录。）
   */
  recordsLocation?: boolean;
}

export function parse(input: string, opts: ParseOptions = {}): Document {
  opts.softBreakAs ??= "br";

  return parser.parse(input, {
    breaks: opts.softBreakAs === "br",
    recordsLocation: opts.recordsLocation ?? false,
  });
}
