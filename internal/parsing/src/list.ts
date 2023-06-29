import * as nodes from "@rotext/nodes";
import type { TextNode } from "@rotext/nodes";

export function buildList(_items: unknown[]): TextNode {
  return nodes.text("(TODO)");
}
