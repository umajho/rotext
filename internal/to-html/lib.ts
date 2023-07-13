import { Element, MixedSlot, RawTextSlot, Root } from "@rotext/nodes";
import { h, VNode, VNodeChildren } from "snabbdom";

export function rootToSnabbdomChildren(root: Root): VNodeChildren {
  return slotToChildren(root.slot);
}
/**
 * XXX: 应由调用者保障传入的元素符合规范（定义的类型）
 * XXX: 小心 XSS，建议用 sanitizer 过滤结果，特别是传入的元素由用户生成时
 */
export function elementToSnabbdom(el: Element): VNode {
  if ("slot" in el) {
    const children = slotToChildren(el.slot);
    let sel: string;
    let classes: Record<string, boolean> | undefined;
    switch (el.type) {
      case "em":
      case "u":
      case "s":
        sel = el.type;
        break;
      case "em.strong":
        sel = "strong";
        break;
      case "em.dotted":
        sel = "em";
        classes = { "em-dotted": true };
        break;
      case "code":
        sel = "code";
        break;
      case "ref-link":
        sel = "span";
        classes = { "ref-link": true };
        break;
      case "P":
      case "QUOTE":
        sel = el.type;
        break;
      case "H":
        sel = `h${el.props.level}`;
        break;
    }
    return classes ? h(sel, { class: classes }, children) : h(sel, children);
  }

  switch (el.type) {
    case "br":
      return h("br");
    case "ruby":
      return h("ruby", [
        h("rb", slotToChildren(el.slots.base)),
        h("rp", String(el.props.p[0])),
        h("rt", slotToChildren(el.slots.text)),
        h("rp", String(el.props.p[1])),
      ]);
    case "dicexp":
      throw new Error("unimplemented");
    case "THEMATIC-BREAK":
      return h("hr");
    case "OL":
    case "UL":
      return h(
        el.type === "OL" ? "ol" : "ul",
        el.items.map((item) => h("li", slotToChildren(item.slot))),
      );
    case "DL":
      return h(
        "dl",
        el.items.map((item) =>
          h(item.type === "DL:T" ? "dt" : "dd", slotToChildren(item.slot))
        ),
      );
    case "TABLE": {
      const children: VNodeChildren = Array(
        el.cells.length + (el.slots?.caption ? 1 : 0),
      );
      let i = 0;
      if (el.slots?.caption) {
        children[i] = h("caption", slotToChildren(el.slots.caption));
        i++;
      }
      for (const row of el.cells) {
        children[i] = h(
          "tr",
          row.map((cell) =>
            h(cell.type === "TABLE:H" ? "th" : "td", slotToChildren(cell.slot))
          ),
        );
        i++;
      }
      return h("table", children);
    }
  }

  el satisfies never;
  throw new Error("unreachable");
}

function slotToChildren(slot: MixedSlot | RawTextSlot): VNodeChildren {
  if (typeof slot === "string") return slot;
  if (slot.length === 1 && typeof slot[0] === "string") return slot[0];

  return slot.map((node) =>
    typeof node === "string" ? node : elementToSnabbdom(node)
  );
}
