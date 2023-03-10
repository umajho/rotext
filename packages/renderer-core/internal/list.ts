import { getCommonPrefix } from "./utils";
import { type V } from "./v";

type RawListItem<VNode> = [string, (VNode | string)[] | string];

type ListKind = "ol" | "ul" | "dl";
type ListItemKind = "li" | "dt" | "dd";

interface ListContainer<VNode> {
  listKind: ListKind;
  items: (ListItem_List<VNode> | ListItem_Other<VNode>)[];
}
interface ListItem_List<VNode> {
  itemKind: ListItemKind;
  list: ListContainer<VNode>;
}
interface ListItem_Other<VNode> {
  itemKind: ListItemKind;
  other: (VNode | string)[] | string;
}

function getListKind(symbol: string) {
  switch (symbol) {
    case "#":
      return "ol";
    case "*":
      return "ul";
    case ";":
    case ":":
      return "dl";
    default:
      throw new Error("unreachable");
  }
}

function getListItemKind(symbol: string) {
  switch (symbol) {
    case "#":
    case "*":
      return "li";
    case ";":
      return "dt";
    case ":":
      return "dd";
    default:
      throw new Error("unreachable");
  }
}

export function buildList<VNode>(
  items: RawListItem<VNode>[],
  v: V<VNode>,
) {
  type LC = ListContainer<VNode>;

  const stack: LC[] = [];
  const done: LC[] = [];
  let lastPrefix = "";

  for (const [itemPrefix, itemContent] of items) {
    const topItemKind = getListItemKind(itemPrefix[itemPrefix.length - 1]);
    const commonPrefix = getCommonPrefix(lastPrefix, itemPrefix);
    const des = lastPrefix.length - commonPrefix.length;
    const ins = itemPrefix.length - commonPrefix.length;

    if (
      des === 1 && ins === 1 && (topItemKind === "dt" || topItemKind === "dd")
    ) {
      // dt, dd 最高处不同不影响
    } else {
      for (let i = 0; i < des; i++) {
        if (stack.length === 1) {
          done.push(stack.pop()!);
        } else {
          stack.pop();
        }
      }
      for (let i = 0; i < ins; i++) {
        const listKind = getListKind(itemPrefix[commonPrefix.length + i]);
        const newList: LC = { listKind, items: [] };
        if (stack.length) {
          const parentItemKind = getListItemKind(
            itemPrefix[commonPrefix.length + i - 1],
          );
          const topList = stack[stack.length - 1];
          topList!.items.push({ itemKind: parentItemKind, list: newList });
        }
        stack.push(newList);
      }
    }

    stack[stack.length - 1].items.push({
      itemKind: topItemKind,
      other: itemContent,
    });
    lastPrefix = itemPrefix;
  }
  done.push(stack[0]!);
  const lists = done.map((tree) => listToVdom(tree, v));
  return lists.length === 1 ? lists[0] : v.fragment(lists);
}

function listToVdom<VNode>(
  list: ListContainer<VNode>,
  v: V<VNode>,
): VNode {
  return v.h(list.listKind, {}, listItemsToVdoms<VNode>(list.items, v));
}

function listItemsToVdoms<VNode>(
  items: ListContainer<VNode>["items"],
  v: V<VNode>,
): (VNode | string)[] | string {
  const acc: ["li" | "dt" | "dd", (VNode | string)[]][] = [];
  for (const [i, item] of items.entries()) {
    if ("other" in item) {
      if (typeof item.other === "string") {
        acc.push([item.itemKind, [item.other]]);
      } else {
        acc.push([item.itemKind, item.other]);
      }
    } else {
      const childList = listItemsToVdoms<VNode>(item.list.items, v);
      const container = v.h(item.list.listKind, {}, childList);
      if (i === 0) {
        acc.push([item.itemKind, [container]]);
      } else {
        acc[acc.length - 1][1].push(container);
      }
    }
  }

  return acc.map((cur) =>
    v.h(cur[0], {}, cur[1].length === 1 ? cur[1][0] : cur[1])
  );
}
