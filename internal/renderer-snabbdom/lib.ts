import { fragment, h, type VNode } from "snabbdom";
import * as core from "@rotext-lite/renderer-core";

export function parse(markup: string, opts?: core.ParseOptions): VNode {
  const v = { h, fragment };

  return core.parse<VNode>(markup, v, opts);
}
