import * as parser from "./src/rotext";

export interface ParseOptions {
  /**
   * 将软换行视为 br 还是空格。
   */
  softBreakAs?: "br" | "space";
}

export function parse(input: string, opts: ParseOptions = {}) {
  opts.softBreakAs ??= "br";

  return parser.parse(input, { breaks: opts.softBreakAs === "br" });
}
