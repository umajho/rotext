import { create, type TextNode } from "@rotext/nodes";

export function buildList(_items: unknown[]): TextNode {
  return create.text("(TODO)");
}
