import { V } from "./internal/v";
import * as rotext from "./internal/rotext.js";

export interface ParseOptions {
  breaks: boolean;
}

export function parse<VNode>(
  markup: string,
  v: V<VNode>,
  opts_: Partial<ParseOptions> = {},
): VNode {
  const opts: ParseOptions = {
    breaks: opts_.breaks ?? false,
  };

  return rotext.parse(markup, { ...opts, v });
}
