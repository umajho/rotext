import { LocationRange } from "peggy";

import {
  BlockElement,
  create,
  DescriptionListItem,
  InlineSlot,
  ListItem,
} from "@rotext/nodes";
import { appendLineToMixedSlot } from "./line";

type ContainerElement =
  & BlockElement
  & { type: "P" | "QUOTE" | "OL" | "UL" | "DL" };
type ContainerItemElement =
  | ListItem
  | DescriptionListItem
  | (BlockElement & { type: "QUOTE" });
interface RootElement {
  type: "ROOT";
  slot: ContainerElement[];
}

interface Item {
  prefix: string;
  line: InlineSlot | undefined;
  location?: LocationRange;
}

interface ItemEx extends Item {
  parentElement: ContainerItemElement | RootElement;
}

type Markup = ">" | "#" | "*" | ";" | ":";
type Group = ">" | "#" | "*" | ";:";

type LocMap = WeakMap<any, LocationRange>;

function recording<T>(
  el: T,
  locMap: LocMap | null,
  location: LocationRange | undefined,
): T {
  locMap?.set(el, location!); // 有 locMap 必定有 location
  return el;
}

export function buildContainers(
  items: Item[],
  breaks: boolean,
  locMap: LocMap | null,
): ContainerElement[] {
  const root: ContainerElement[] = [];
  const rootElement: RootElement = { type: "ROOT", slot: root };

  let currentItems: ItemEx[] = items
    .map((item, i) => ({
      ...item,
      parentElement: rootElement,
      nothingAbove: i === 0,
    }));

  const maxDepth = Math.max(...items.map((item) => item.prefix.length)) + 1;
  for (let currentDepth = 1; currentDepth <= maxDepth; currentDepth++) {
    const deeperItems: ItemEx[] = [];
    // 上一项是什么：
    // - `"container"`：容器，也即处理的是 `item.prefix`；
    // - `"line"`：行内内容，也即处理的是 `item.line`；
    // - `null`：没有上一行（现在处理的是第一项）。
    let last:
      | {
        type: "container";
        parentEl: ContainerItemElement; // 每行可能变化
        group: Group; // 与先前的组不同的时候变化
        el: ContainerElement; // 同上
        itemEl: ContainerItemElement; // 每行变化
      }
      | {
        type: "line";
        parentEl: ContainerItemElement; // 每行可能变化
        isFirst: boolean; // 前两行变化
        isBlank: boolean; // 每行变化
      }
      | { type: null } = { type: null };

    for (const item of currentItems) {
      item.line ??= [];

      if (currentDepth > item.prefix.length) { // 只剩 `item.line`
        const parentEl = item.parentElement;
        if (
          "type" in parentEl && parentEl.type === "ROOT"
        ) { // 不可能存在没有 prefix 的情况
          throw new Error("unreachable");
        }

        const isFirst = last.type === null
          ? true
          : (last.parentEl !== parentEl);
        const isBlank = item.line.length === 0 ||
          (item.line.length === 1 && typeof item.line[0] === "string" &&
            /^ \t*$/.test(item.line[0]));
        if (!isBlank) {
          const paragraphOpt = ((): "no" | "new" | "default" => {
            if ("type" in parentEl && parentEl.type === "QUOTE") {
              return last.type === "line" && last.isBlank ? "new" : "default";
            }
            return isFirst
              ? "no"
              : ((last.type === "line" && (last.isFirst || last.isBlank))
                ? "new"
                : "default");
          })();
          appendLineToItem(
            parentEl,
            item.line,
            {
              paragraph: paragraphOpt,
              breaks,
              recording: locMap
                ? <T>(el: T) => recording(el, locMap, item.location!)
                : undefined,
            },
          );
        }

        last = { type: "line", parentEl, isFirst, isBlank };
      } else { // 尚有 `item.prefix` 要处理
        const markup = getMarkup(item.prefix, currentDepth);
        const group = getGroup(markup);
        const sameParentWithLast = last.type === "container" &&
          last.parentEl === item.parentElement;

        // NOTE: 先前也考虑过 “在非最后一层允许同组合并为一项” 以兼容 wikitext，
        //       但最后还是选择了一致性。
        if (sameParentWithLast && last.type === "container" && group === ">") {
          // 本行下一层的项与上一行下一层的项同属这一层的一项

          deeperItems.push({ ...item, parentElement: last.itemEl });
        } else {
          let el: ContainerElement;
          if (
            sameParentWithLast && last.type === "container" &&
            group === last.group
          ) {
            el = last.el;
          } else {
            el = createContainer(group, locMap, item.location);
            appendContainerToItem(item.parentElement, el);
          }
          const itemEl = createAndAppendItemToContainer(
            el,
            markup,
            locMap,
            item.location,
          );
          deeperItems.push({ ...item, parentElement: itemEl });
          last = {
            type: "container",
            parentEl: item.parentElement,
            group,
            el,
            itemEl,
          };
        }
      }
    }

    currentItems = deeperItems;
  }

  return root;
}

function createContainer(
  group: Group,
  locMap: LocMap | null,
  location: LocationRange | undefined,
): ContainerElement {
  function _createContainer(group: Group): ContainerElement {
    switch (group) {
      case ">":
        return create.QUOTE([]);
      case "#":
        return create.LIST("O", []);
      case "*":
        return create.LIST("U", []);
      case ";:":
        return create.DL([]);
    }
  }

  const container = _createContainer(group);
  return recording(container, locMap, location);
}

/**
 * 如果是第一行，则直接将 line 保持 inlineSlot 追加；
 * 如果不是第一行，但上一行是第一行，或者上一行是空行，则追加新的段落，然后再将 line 放入段落；
 * 否则，追加到之前的段落中。
 *
 * 第二行之所以不会并入第一行，是为了与表格的行为（如 `{|\n|foo\nbar\n|}`）一致，
 * 此行为来自 wikitext。
 *
 * 当 currentIsFrist 为真时，lastIsFirstOrBlank 无关紧要，应设为 null。
 */
function appendLineToItem(
  target: ContainerItemElement,
  line: InlineSlot,
  opts: {
    paragraph: "default" | "new" | "no";
    breaks: boolean;
    recording?: <T>(el: T) => T;
  },
) {
  appendLineToMixedSlot(target.slot, line, opts);
}

function appendContainerToItem(
  target: ContainerItemElement,
  el: ContainerElement,
) {
  target.slot.push(el);
}

function createAndAppendItemToContainer(
  target: ContainerElement,
  markup: Markup,
  locMap: LocMap | null,
  location: LocationRange | undefined,
): ContainerItemElement {
  function _createAndAppendItemToContainer(
    target: ContainerElement,
    markup: Markup,
  ): ContainerItemElement {
    switch (markup) {
      case "#":
      case "*": {
        if (target.type !== (markup === "#" ? "OL" : "UL")) {
          throw new Error("unreachable");
        }
        const itemEl = create.LIST$item([]);
        target.items.push(itemEl);
        return itemEl;
      }
      case ";":
      case ":": {
        if (target.type !== "DL") throw new Error("unreachable");
        const itemEl = create.DL$item(markup === ";" ? "T" : "D", []);
        target.items.push(itemEl);
        return itemEl;
      }
      case ">": {
        if (target.type !== "QUOTE") throw new Error("unreachable");
        return target;
      }
      default:
        throw new Error("unreachable");
    }
  }

  const item = _createAndAppendItemToContainer(target, markup);
  return recording(item, locMap, location);
}

function getMarkup(prefix: string, depth: number): Markup {
  const markup = prefix[depth - 1];
  if (
    markup !== ">" && markup !== "#" && markup !== "*" && markup !== ";" &&
    markup !== ":"
  ) {
    throw new Error("unreachable");
  }
  return markup;
}

function getGroup(markup: Markup): Group {
  if (markup === ">" || markup === "#" || markup === "*") return markup;
  if (markup === ";" || markup === ":") return ";:";
  throw new Error("unreachable");
}
