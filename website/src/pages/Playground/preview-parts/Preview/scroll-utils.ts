import { binarySearch } from "../../../../utils/algorithm";

import {
  ElementLocationPair,
  LookupList,
  LookupListRaw,
  ScrollLocal,
} from "./internal-types";

export function getScrollLocalByY(
  lookupList: LookupList,
  y: number,
  globalOffsetBottom: number,
): ScrollLocal {
  const localIndex = binarySearch(lookupList, (item, i) => {
    if (item.offsetTop > y) return "less";

    const nextItem = lookupList[i + 1];
    if (!nextItem || y < nextItem.offsetTop) return true;
    return "greater";
  }) ?? 0;
  const localItem = lookupList[localIndex]!;

  const offsetTop = localItem.offsetTop;
  const offsetBottom = getOffsetBottom(
    lookupList,
    localIndex,
    globalOffsetBottom,
  );

  const progress = (y - offsetTop) / (offsetBottom - offsetTop);

  return {
    indexInLookupList: localIndex,
    progress,
  };
}

export function getScrollLocalByLine(
  lookupList: LookupList,
  line: number,
): ScrollLocal {
  const localIndex = binarySearch(lookupList, (item, i) => {
    if (item.location.start.line > line) return "less";

    const nextItem = lookupList[i + 1];
    if (!nextItem || line < nextItem.location.start.line) return true;
    return "greater";
  }) ?? 0;
  const localItem = lookupList[localIndex]!;

  const startLine = localItem.location.start.line;
  const endLine = getEndLine(
    lookupList,
    localIndex,
    localItem.location.end.line,
  );

  const progress = (line - startLine) / (endLine - startLine + 1);

  return {
    indexInLookupList: localIndex,
    progress,
  };
}

export function scrollLocalToLine(local: ScrollLocal, list: LookupList) {
  const item = list[local.indexInLookupList]!;

  const startLine = item.location.start.line;
  const endLine = getEndLine(
    list,
    local.indexInLookupList,
    item.location.end.line,
  );

  return Math.max(startLine + (endLine - startLine + 1) * local.progress, 1);
}

function scrollLocalToScrollTop(
  local: ScrollLocal,
  list: LookupList,
  globalOffsetBottom: number,
) {
  const item = list[local.indexInLookupList];
  if (!item) return;

  const offsetTop = item.offsetTop;
  const offsetBottom = getOffsetBottom(
    list,
    local.indexInLookupList,
    globalOffsetBottom,
  );

  return offsetTop + (offsetBottom - offsetTop) * local.progress;
}

/**
 * @returns
 * - `"scrolled"`: 正常滚动。
 * - `"adjusted"`: 只是调整一下位置。（内容底部增减内容，且满足一定条件，导致顶部位置改变场合。）
 * - `"untouched"`: 没有动。
 */
export function scrollToLine(
  line: number,
  lookupList: LookupList,
  scrollContainerEl: HTMLElement,
  wasAtBottom: () => boolean,
  setWasAtBottom: (v: boolean) => void,
  opts?: { triggeredBy?: "text-changes" },
): "scrolled" | "untouched" | "adjusted" {
  const scrollLocal = getScrollLocalByLine(lookupList, line);

  const maxScrollTop = scrollContainerEl.scrollHeight -
    scrollContainerEl.offsetHeight;

  const scrollTop = Math.min(
    scrollLocalToScrollTop(
      scrollLocal,
      lookupList,
      scrollContainerEl.scrollHeight,
    ) ?? 0,
    maxScrollTop,
  );

  // 原先在底部，现在却往下滚动，代表下方新增了内容，而现在只是因为
  // “之前编辑器的位置已经越过了预览对应能滚动到的位置（最底部），现在预览的最底部增加了”
  // 而调整预览的滚动位置。
  const adjusting = wasAtBottom() && scrollTop > scrollContainerEl.scrollTop;
  if (opts?.triggeredBy === "text-changes" && !adjusting) return "untouched";

  scrollContainerEl.scrollTo({ top: scrollTop, behavior: "instant" });
  setWasAtBottom(isAtBottom(scrollContainerEl));

  return adjusting
    ? "adjusted"
    : (scrollTop < maxScrollTop ? "scrolled" : "untouched");
}

export function isAtBottom(
  scrollContainerEl: HTMLElement,
) {
  // FIXME: 由于未知原因，Chrome 上 scrollTop+offsetHeight 与 scrollHeight
  //        差了整整 0.5（而 Safari 没有这个问题），这里暂时直接这么补上。
  return scrollContainerEl.scrollTop + scrollContainerEl.offsetHeight + 0.5 >=
    scrollContainerEl.scrollHeight;
}

export function roastLookupList(raw: LookupListRaw, rootClass: string) {
  if (!raw.length) return [];

  // 按理来讲应该已经是按起始行数排序的了，不过以免万一就再排序一次。
  // 其实原本还会保证越深的元素排在越后面，不过后面的操作不用考虑这件事。
  raw.sort((a, b) => a.location.start.line - b.location.start.line);

  const rootElementViewportOffsetTop = (() => {
    let el = raw[0]!.element;
    while (!el.classList.contains(rootClass)) {
      el = el.parentElement!;
    }
    return el.getBoundingClientRect().top;
  })();

  // 为每一项就地设置位移高度
  const roasted = raw as LookupList;
  for (const item of roasted) {
    const itemElementViewportOffsetTop =
      item.element.getBoundingClientRect().top;
    item.offsetTop = itemElementViewportOffsetTop -
      rootElementViewportOffsetTop;
  }

  // 根据规则，在起始行数相同的项有多项时只保留一项
  const reduced = roasted.reduce((acc, cur) => {
    if (!acc) return [cur];
    const last = acc[acc.length - 1]!;

    // NOTE: 有可能两个元素的起始行数、高度都一样，
    //       这时用哪个都一样，因为用不到更细的信息。
    if (last.location.start.line === cur.location.start.line) {
      if (last.offsetTop < cur.offsetTop) return acc;
      if (last.element.offsetHeight <= cur.element.offsetHeight) return acc;
      acc[acc.length - 1] = cur;
      return acc;
    }
    acc.push(cur);
    return acc;
  }, null as ElementLocationPair[] | null) ?? [];

  return reduced;
}

function getOffsetBottom(
  lookupList: LookupList,
  localIndex: number,
  globalOffsetBottom: number,
): number {
  const nextItem: ElementLocationPair | undefined = lookupList[localIndex + 1];
  return nextItem ? nextItem.offsetTop : globalOffsetBottom;
}

function getEndLine(
  lookupList: LookupList,
  localIndex: number,
  localEndLine: number,
): number {
  const nextItem = lookupList[localIndex + 1];
  return nextItem ? nextItem.location.start.line - 1 : localEndLine;
}
