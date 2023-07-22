import "./mod.scss";

import {
  Component,
  createEffect,
  createMemo,
  createSignal,
  on,
  onMount,
  Setter,
  Show,
} from "solid-js";

import { classModule, h, init, Module, styleModule, VNode } from "snabbdom";

import { parse } from "@rotext/parsing";
import { toSnabbdomChildren } from "@rotext/to-html";

import { debounceEventHandler } from "../../../utils/mod";

import { EditorStore, TopLine } from "../../../hooks/editor-store";

import { LookupList, LookupListRaw } from "./internal-types";
import * as ScrollUtils from "./scroll-utils";

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
    setLookupList(ScrollUtils.roastLookupList(_raw, ROOT_CLASS));
  }
  createEffect(on([lookupListRaw], roastLookupList));
  onMount(() => {
    new ResizeObserver(roastLookupList).observe(scrollContainerEl);
  });

  let outputContainerEl: HTMLDivElement;

  //==== 文档渲染 ====
  onMount(() => {
    createRendering({
      text: () => props.store.text,
      setLookupListRaw,
      setParsingTimeText: (v) => props.setParsingTimeText(v),
      setErr,
    }, {
      outputContainer: outputContainerEl,
    });
  });

  //==== 滚动同步 ====
  const [scrollHandler, setScrollHandler] = createSignal<(ev: Event) => void>();
  const debouncedScrollHandler = createMemo(() =>
    scrollHandler() && debounceEventHandler(scrollHandler())
  );
  onMount(() => {
    const { handleScroll } = createScrollSyncing({
      text: () => props.store.text,
      topLine: () => props.store.topLine,
      setTopLine: (v) => props.store.topLine = v,
      lookupList,
    }, {
      scrollContainer: scrollContainerEl,
      outputContainer: outputContainerEl,
    });
    setScrollHandler(() => handleScroll);
  });

  //==== 组件 ====
  return (
    <div
      class={`relative previewer-background overflow-y-auto ${
        props.class ?? ""
      }`}
      ref={scrollContainerEl}
      onScroll={(ev) => debouncedScrollHandler()(ev)}
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

/**
 * 用于处理文档渲染。
 */
function createRendering(
  props: {
    text: () => string;
    setLookupListRaw: (v: LookupListRaw) => void;
    setParsingTimeText(v: string): void;
    setErr: (v: Error) => void;
  },
  els: {
    outputContainer: HTMLElement;
  },
) {
  let patch: ReturnType<typeof init>;
  let lastNode: HTMLElement | VNode;

  const {
    module: locationModule,
  } = createLocationModule(props.setLookupListRaw);

  onMount(() => {
    const outputEl = document.createElement("div");
    els.outputContainer.appendChild(outputEl);

    patch = init(
      [classModule, styleModule, locationModule],
      undefined,
      { experimental: { fragments: true } },
    );
    lastNode = outputEl;
  });

  createEffect(on([props.text], () => {
    props.setErr(null);
    try {
      const parsingStart = performance.now();
      const doc = parse(props.text(), {
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
      props.setErr(e);
    }
  }));
}

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
 * 用于处理滚动同步。
 */
function createScrollSyncing(
  props: {
    text: () => string;
    topLine: () => TopLine;
    setTopLine: (v: TopLine) => void;
    lookupList: () => LookupList;
  },
  els: {
    scrollContainer: HTMLElement;
    outputContainer: HTMLElement;
  },
) {
  let pendingAutoScrolls = 0;

  {
    function scrollToTopLine() {
      const topLineData = props.topLine();
      if (!topLineData.setFrom || topLineData.setFrom === "preview") {
        return;
      }

      const _lookupList = props.lookupList();
      if (!_lookupList.length) return;

      const scrollResult = ScrollUtils.scrollToLine(
        topLineData.number,
        _lookupList,
        els.scrollContainer,
      );
      if (scrollResult === "scrolled") {
        pendingAutoScrolls++;
      }
    }

    createEffect(
      on([() => props.topLine(), () => props.text()], scrollToTopLine),
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

    const _lookupList = props.lookupList();
    if (!_lookupList?.length) return;

    const _baselineY = els.scrollContainer.scrollTop -
      els.outputContainer.offsetTop;

    const scrollLocal = ScrollUtils.getScrollLocalByY(
      _lookupList,
      _baselineY,
      els.outputContainer.offsetHeight,
    );
    const line = ScrollUtils.scrollLocalToLine(scrollLocal, _lookupList);

    {
      // 配合编辑器的 “滚动过最后一行” 功能。
      // 否则在当编辑文本，导致预览的高度改变时，编辑器的滚动位置也会复位。
      // FIXME: 由于未知原因，Chrome 上 scrollTop+offsetHeight 与 scrollHeight
      //        差了整整 0.5（而 Safari 没有这个问题），这里暂时直接这么补上。
      const atBottom =
        els.scrollContainer.scrollTop + els.scrollContainer.offsetHeight +
            0.5 >=
          els.scrollContainer.scrollHeight;
      const lastTopLine = props.topLine();
      if (
        atBottom && lastTopLine.setFrom === "editor" &&
        line <= lastTopLine.number
      ) {
        return;
      }
    }

    props.setTopLine({ number: line, setFrom: "preview" });
  }

  return { handleScroll };
}
