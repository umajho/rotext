import "./preview.scss";

import {
  Component,
  createEffect,
  createSignal,
  on,
  onMount,
  Setter,
  Show,
} from "solid-js";

import { classModule, h, init, Module, styleModule, VNode } from "snabbdom";

import { parse } from "@rotext/parsing";
import { toSnabbdomChildren } from "@rotext/to-html";

import { debounceEventHandler } from "../../utils/mod";

import { EditorStore } from "../../hooks/editor-store";

const ROOT_CLASS = "rotext-preview-root";

const Preview: Component<
  {
    store: EditorStore;
    class?: string;
    setParsingTimeText: Setter<string>;
    onThrowInParsing: (thrown: unknown) => void;
  }
> = (props) => {
  let scrollContainerEl: HTMLDivElement;

  const [err, setErr] = createSignal<Error>(null);

  const [lookupListRaw, setLookupListRaw] = createSignal<LookupList>();
  const [lookupList, setLookupList] = createSignal<LookupList>();
  function roastLookupList() {
    const _raw = lookupListRaw();
    if (!_raw) return;
    setLookupList(ScrollSyncUtils.roastLookupList(_raw, ROOT_CLASS));
  }
  createEffect(on([lookupListRaw], roastLookupList));
  onMount(() => {
    new ResizeObserver(roastLookupList).observe(scrollContainerEl);
  });

  //==== 文档渲染 ====

  let outputContainerEl: HTMLDivElement;
  let patch: ReturnType<typeof init>;
  let lastNode: HTMLElement | VNode;

  const {
    module: locationModule,
  } = createLocationModule(setLookupListRaw);

  onMount(() => {
    const outputEl = document.createElement("div");
    outputContainerEl.appendChild(outputEl);

    patch = init(
      [classModule, styleModule, locationModule],
      undefined,
      { experimental: { fragments: true } },
    );
    lastNode = outputEl;
  });

  createEffect(on([() => props.store.text], () => {
    setErr(null);
    try {
      const parsingStart = performance.now();
      const doc = parse(props.store.text, {
        softBreakAs: "br",
        recordsLocation: true,
      });
      const vChildren = toSnabbdomChildren(doc);
      props.setParsingTimeText(
        `${+(performance.now() - parsingStart).toFixed(3)}ms`,
      );

      const classMap = { "relative": true };
      classMap[ROOT_CLASS] = true;
      const vNode = h("article", { class: classMap }, vChildren);

      patch(lastNode, vNode);
      lastNode = vNode;
    } catch (e) {
      if (!(e instanceof Error)) {
        e = new Error(e);
      }
      setErr(e);
    }
  }));

  //==== 滚动同步 ====

  // const [scrollLocal, setScrollLocal] = createSignal<ScrollLocal>();

  let pendingAutoScrolls = 0;

  {
    function scrollToTopLine() {
      const topLineData = props.store.topLine;
      if (!topLineData.setFrom || topLineData.setFrom === "preview") {
        return;
      }

      const _lookupList = lookupList();
      if (!_lookupList.length) return;

      const scrollResult = ScrollSyncUtils.scrollToLine(
        topLineData.number,
        _lookupList,
        scrollContainerEl,
      );
      if (scrollResult === "scrolled") {
        pendingAutoScrolls++;
      }
    }

    createEffect(
      on([() => props.store.topLine, () => props.store.text], scrollToTopLine),
    );
  }

  /**
   * @param scrollContainerEl 滚动内容的容器元素。
   *  除了预览内容之外，还包含可能存在的错误展示等内容。
   */
  function handleScroll(_ev: Event) {
    if (pendingAutoScrolls > 0) {
      pendingAutoScrolls = Math.max(pendingAutoScrolls - 1, 0);
      return;
    }

    const _lookupList = lookupList();
    if (!_lookupList?.length) return;

    const _baselineY = scrollContainerEl.scrollTop -
      outputContainerEl.offsetTop;

    const scrollLocal = ScrollSyncUtils.getScrollLocalByY(
      _lookupList,
      _baselineY,
      outputContainerEl.offsetHeight,
    );
    const line = ScrollSyncUtils.scrollLocalToLine(scrollLocal, _lookupList);

    {
      // 配合编辑器的 “滚动过最后一行” 功能。
      // 否则在当编辑文本，导致预览的高度改变时，编辑器的滚动位置也会复位。
      // FIXME: 由于未知原因，Chrome 上 scrollTop+offsetHeight 与 scrollHeight
      //        差了整整 0.5（而 Safari 没有这个问题），这里暂时直接这么补上。
      const atBottom =
        scrollContainerEl.scrollTop + scrollContainerEl.offsetHeight + 0.5 >=
          scrollContainerEl.scrollHeight;
      const lastTopLine = props.store.topLine;
      if (
        atBottom && lastTopLine.setFrom === "editor" &&
        line <= lastTopLine.number
      ) {
        return;
      }
    }

    props.store.topLine = { number: line, setFrom: "preview" };
  }

  //==== 组件 ====
  return (
    <div
      class={`relative previewer-background overflow-y-auto ${
        props.class ?? ""
      }`}
      ref={scrollContainerEl}
      onScroll={debounceEventHandler(handleScroll)}
    >
      <Show when={err()}>
        <ErrorAlert error={err()} showsStack={true} />
      </Show>
      <div
        class={"" +
          "relative " + // 作为计算元素高度位移的锚点
          "self-center mx-auto " + // 保持居中，以及撑起父元素
          "break-all prose previewer " + // 内容的外观样式
          ""}
      >
        <div ref={outputContainerEl} />
      </div>
    </div>
  );
};
export default Preview;

const ErrorAlert: Component<{
  error: Error;
  showsStack: boolean;
}> = (props) => {
  return (
    <div class="sticky top-0 z-10 max-h-32 alert alert-error shadow-lg overflow-scroll">
      <div class="text-xs">
        <code class="whitespace-pre">
          {props.error.message}
          <Show when={props.showsStack && props.error["stack"]}>
            <hr />
            {props.error["stack"]}
          </Show>
        </code>
      </div>
    </div>
  );
};

interface LocationData {
  start: { line: number };
  end: { line: number };
}
interface ElementLocationPair {
  element: HTMLElement;
  location: LocationData;
  offsetTop: number;
}
type LookupList = ElementLocationPair[];
type LookupListRaw = Omit<ElementLocationPair, "offsetTop">[];

function createLocationModule(
  setLookupListRaw: (view: LookupListRaw) => void,
): { module: Module } {
  let loookupListRaw!: LookupListRaw;

  const module = {
    pre: () => {
      loookupListRaw = [];
    },
    create: (_oldVNode: VNode, vnode: VNode) => {
      if (vnode.data.location) {
        const el = vnode.elm as HTMLElement;
        loookupListRaw.push({
          element: el,
          location: vnode.data.location,
        });
      }
    },
    update: (oldVNode: VNode, vnode: VNode) => {
      module.create(oldVNode, vnode);
    },
    post: () => {
      setLookupListRaw(loookupListRaw);
    },
  };

  return { module };
}

/**
 * baseline 所穿过的元素、到达下一个这样的元素的进度，以及这个元素对应于原始输入的行数。
 */
interface ScrollLocal {
  indexInLookupList: number;
  progress: number;
}
const ScrollSyncUtils = {
  getScrollLocalByY(
    lookupList: LookupList,
    y: number,
    globalOffsetBottom: number,
  ): ScrollLocal {
    const localIndex = ScrollSyncUtils.binarySearch(lookupList, (item, i) => {
      if (item.offsetTop > y) return "less";

      const nextItem = lookupList[i + 1];
      if (!nextItem || y < nextItem.offsetTop) return true;
      return "greater";
    }) ?? 0;
    const localItem = lookupList[localIndex];

    const offsetTop = localItem.offsetTop;
    const offsetBottom = ScrollSyncUtils.getOffsetBottom(
      lookupList,
      localIndex,
      globalOffsetBottom,
    );

    const progress = (y - offsetTop) / (offsetBottom - offsetTop);

    return {
      indexInLookupList: localIndex,
      progress,
    };
  },

  getScrollLocalByLine(lookupList: LookupList, line: number): ScrollLocal {
    const localIndex = ScrollSyncUtils.binarySearch(lookupList, (item, i) => {
      if (item.location.start.line > line) return "less";

      const nextItem = lookupList[i + 1];
      if (!nextItem || line < nextItem.location.start.line) return true;
      return "greater";
    }) ?? 0;
    const localItem = lookupList[localIndex];

    const startLine = localItem.location.start.line;
    const endLine = ScrollSyncUtils.getEndLine(
      lookupList,
      localIndex,
      localItem.location.end.line,
    );

    const progress = (line - startLine) / (endLine - startLine + 1);

    return {
      indexInLookupList: localIndex,
      progress,
    };
  },

  scrollLocalToLine(local: ScrollLocal, list: LookupList) {
    const item = list[local.indexInLookupList];

    const startLine = item.location.start.line;
    const endLine = ScrollSyncUtils.getEndLine(
      list,
      local.indexInLookupList,
      item.location.end.line,
    );

    return Math.max(startLine + (endLine - startLine + 1) * local.progress, 1);
  },

  scrollLocalToScrollTop(
    local: ScrollLocal,
    list: LookupList,
    globalOffsetBottom: number,
  ) {
    const item = list[local.indexInLookupList];
    if (!item) return;

    const offsetTop = item.offsetTop;
    const offsetBottom = ScrollSyncUtils.getOffsetBottom(
      list,
      local.indexInLookupList,
      globalOffsetBottom,
    );

    return offsetTop + (offsetBottom - offsetTop) * local.progress;
  },

  scrollToLine(
    line: number,
    lookupList: LookupList,
    scrollContainerEl: HTMLElement,
  ): "scrolled" | "untouched" | "adjusted" {
    const scrollLocal = ScrollSyncUtils.getScrollLocalByLine(
      lookupList,
      line,
    );

    const maxScrollTop = scrollContainerEl.scrollHeight -
      scrollContainerEl.offsetHeight;

    const scrollTop = Math.min(
      ScrollSyncUtils.scrollLocalToScrollTop(
        scrollLocal,
        lookupList,
        scrollContainerEl.scrollHeight,
      ),
      maxScrollTop,
    );

    if (scrollTop < maxScrollTop || scrollTop > scrollContainerEl.scrollTop) {
      scrollContainerEl.scrollTo({ top: scrollTop, behavior: "instant" });
      return scrollTop < maxScrollTop ? "scrolled" : "adjusted";
    }
    return "untouched";
  },

  roastLookupList(raw: LookupListRaw, rootClass: string) {
    // 按理来讲应该已经是按起始行数排序的了，不过以免万一就再排序一次。
    // 其实原本还会保证越深的元素排在越后面，不过后面的操作不用考虑这件事。
    raw.sort((a, b) => a.location.start.line - b.location.start.line);

    const [rootElementViewportOffsetTop] = (() => {
      if (!raw.length) return [null, null];
      let el = raw[0].element;
      while (!el.classList.contains(rootClass)) {
        el = el.parentElement;
      }
      return [el.getBoundingClientRect().top];
    })();

    // 为每一项就地设置位移高度
    const roasted = raw as ElementLocationPair[];
    for (const item of roasted) {
      const itemElementViewportOffsetTop =
        item.element.getBoundingClientRect().top;
      item.offsetTop = itemElementViewportOffsetTop -
        rootElementViewportOffsetTop;
    }

    // 根据规则，在起始行数相同的项有多项时只保留一项
    const reduced = roasted.reduce((acc, cur) => {
      if (!acc) return [cur];
      const last = acc[acc.length - 1];

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
  },

  binarySearch<T>(
    list: Array<T>,
    predicate: (item: T, i: number) => true | "greater" | "less",
  ): number | null {
    let l = 0, h = list.length - 1;

    while (true) {
      if (h < l) return null;
      const i = ((h - l + 1) >> 2) + l;
      const item = list[i];
      const p = predicate(item, i);
      if (p === true) return i;
      if (p === "greater") {
        l = i + 1;
      } else {
        h = i - 1;
      }
    }
  },

  getOffsetBottom(
    lookupList: LookupList,
    localIndex: number,
    globalOffsetBottom: number,
  ): number {
    const nextItem: ElementLocationPair | undefined =
      lookupList[localIndex + 1];
    return nextItem ? nextItem.offsetTop : globalOffsetBottom;
  },

  getEndLine(
    lookupList: LookupList,
    localIndex: number,
    localEndLine: number,
  ): number {
    const nextItem = lookupList[localIndex + 1];
    return nextItem ? nextItem.location.start.line - 1 : localEndLine;
  },
};
