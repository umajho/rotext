import { Document, Element, MixedSlot, RawTextSlot } from "@rotext/nodes";
import { h, VNode, VNodeChildren } from "snabbdom";
import { LocationRange } from "peggy";

type LocMap = WeakMap<any, LocationRange>;

export interface CustomElementTagNameMap {
  "scratch-off": string;
  "ref-link": string;
  "dicexp-preview": string;
}

export function toSnabbdomChildren(
  doc: Document,
  opts: {
    customElementTagNameMap: CustomElementTagNameMap;
  },
): VNodeChildren {
  const locMap = doc.metadata?.locMap as LocMap | undefined;
  return slotToChildren(doc.slot, {
    locMap,
    customElementTagNameMap: opts.customElementTagNameMap,
  });
}
/**
 * XXX: 应由调用者保障传入的元素符合规范（定义的类型）
 * XXX: 小心 XSS，建议用 sanitizer 过滤结果，特别是传入的元素由用户生成时
 */
export function elementToSnabbdom(
  el: Element,
  opts: {
    locMap: LocMap | undefined;
    customElementTagNameMap: CustomElementTagNameMap;
  },
): VNode {
  let children: VNodeChildren | undefined;
  const location = getLocationData(opts.locMap, el);

  if ("slot" in el) {
    let sel: string;
    let classes: Record<string, boolean> | undefined;
    let attrs: Record<string, string> | undefined;
    switch (el.type) {
      // case "em":
      // case "u":
      case "s":
        sel = el.type;
        break;
      case "em.strong":
        sel = "strong";
        break;
      // case "em.dotted":
      //   sel = "em";
      //   classes = { "em-dotted": true };
      //   break;
      case "spoiler":
        sel = opts.customElementTagNameMap["scratch-off"];
        children = h(
          "span",
          { attrs: { "slot": "content" } },
          slotToChildren(el.slot, opts),
        );
        break;
      case "code":
        sel = "code";
        break;
      case "ref-link":
        sel = opts.customElementTagNameMap["ref-link"];
        // fallback
        children = h(
          "span",
          { class: { "widget-prime": true } },
          `>>${el.slot}`,
        );
        attrs = { address: el.slot };
        break;
      case "P":
        sel = el.type;
        break;
      case "QUOTE":
        sel = "blockquote";
        break;
      case "H":
        sel = `h${el.props.level}`;
        break;
    }

    children ??= slotToChildren(el.slot, opts);
    const data = (classes || location || attrs)
      ? { class: classes, location, attrs }
      : null;

    return h(sel, data, children);
  }

  switch (el.type) {
    case "br":
      return h("br", location ? { location } : null);
    case "ruby":
      return h("ruby", location ? { location } : null, [
        h("rb", slotToChildren(el.slots.base, opts)),
        h("rp", String(el.props.p[0])),
        h("rt", slotToChildren(el.slots.text, opts)),
        h("rp", String(el.props.p[1])),
      ]);
    case "hyperlink":
      if ("props" in el && "auto" in el.props && el.props.auto) {
        return h("a", {
          attrs: {
            href: el.slots.href,
            target: "_blank",
            rel: "noopener noreferrer",
          },
        }, el.slots.href);
      } else {
        // el satisfies never;
        throw new Error("unreachable");
      }
    case "dicexp": {
      const attrs = {
        code: el.slots.code,
        ...(el.slots.assignTo ? { "assign-to": el.slots.assignTo } : {}),
      };
      // fallback
      const children: VNodeChildren = h(
        "span",
        { class: { "widget-prime": true } },
        "[" +
          `${el.slots.assignTo ? `@${el.slots.assignTo}` : ""}` +
          `=${el.slots.code}` +
          "]",
      );
      // TODO: 根据附加数据决定标签名（`…-preview` vs `…-result`？）
      return h(
        opts.customElementTagNameMap["dicexp-preview"],
        { attrs },
        children,
      );
    }
    case "THEMATIC-BREAK":
      return h("hr", location ? { location } : null);
    case "OL":
    case "UL":
      return h(
        el.type === "OL" ? "ol" : "ul",
        location ? { location } : null,
        el.items.map((item) => {
          const itemLocation = getLocationData(opts.locMap, item);
          return h(
            "li",
            itemLocation ? { location: itemLocation } : null,
            slotToChildren(item.slot, opts),
          );
        }),
      );
    case "DL":
      return h(
        "dl",
        location ? { location } : null,
        el.items.map((item) => {
          const itemLocation = getLocationData(opts.locMap, item);
          const children = slotToChildren(item.slot, opts);
          return h(
            item.type === "DL:T" ? "dt" : "dd",
            itemLocation ? { location: itemLocation } : null,
            children,
          );
        }),
      );
    case "TABLE": {
      const children: VNodeChildren = Array(
        el.cells.length + (el.slots?.caption ? 1 : 0),
      );
      let i = 0;
      if (el.slots?.caption) {
        children[i] = //
          h("caption", slotToChildren(el.slots.caption, opts));
        i++;
      }
      for (const row of el.cells) {
        const rowLocation = getLocationData(opts.locMap, row);

        children[i] = h(
          "tr",
          rowLocation ? { location: rowLocation } : null,
          row.map((cell) => {
            const cellLocation = getLocationData(opts.locMap, cell);
            const children = slotToChildren(cell.slot, opts);
            return h(
              cell.type === "TABLE:H" ? "th" : "td",
              cellLocation ? { location: cellLocation } : null,
              children,
            );
          }),
        );
        i++;
      }
      return h("table", location ? { location } : null, children);
    }
  }

  el satisfies never;
  throw new Error("unreachable");
}

function slotToChildren(
  slot: MixedSlot | RawTextSlot,
  opts: {
    locMap: LocMap | undefined;
    customElementTagNameMap: CustomElementTagNameMap;
  },
): VNodeChildren {
  if (typeof slot === "string") return slot;
  if (slot.length === 1 && typeof slot[0] === "string") return slot[0];

  return slot.map((node) =>
    typeof node === "string" ? node : elementToSnabbdom(node, opts)
  );
}

interface LocationData {
  start: { line: number };
  end: { line: number };
}

function getLocationData(
  locMap: LocMap | undefined,
  key: any,
): LocationData | undefined {
  if (!locMap) return undefined;
  const location = locMap.get(key);
  if (!location) return undefined;
  return {
    start: { line: location.start.line },
    end: { line: location.end.line },
  };
}
