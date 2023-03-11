import { Fragment, h, type VNode } from "vue";
import * as core from "rotext-renderer-core";

export function parse(markup: string, opts?: core.ParseOptions): VNode {
  const v = { h, fragment: (nodes: VNode[]) => h(Fragment, {}, nodes) };

  return core.parse<VNode>(markup, v, opts);
}
