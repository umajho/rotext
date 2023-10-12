import "./tuan-prose.scss";

import {
  Component,
  createEffect,
  createSignal,
  JSX,
  on,
  onMount,
  Setter,
  Show,
} from "solid-js";

import {
  Classes,
  classModule,
  h,
  init,
  Module,
  propsModule,
  styleModule,
  VNode,
} from "snabbdom";

import { parse } from "@rotext/parsing";
import { toSnabbdomChildren } from "@rotext/to-html";

import { registerRoWidgetOwner } from "@rotext/solid-components/internal";

import { ErrorAlert } from "./ui";
import { Loading } from "../../../ui";

import { debounceEventHandler } from "../../../../utils/mod";

import {
  ActiveLines,
  EditorStore,
  TopLine,
} from "../../../../hooks/editor-store";

import { LookupList, LookupListRaw } from "./internal-types";
import * as ScrollUtils from "./scroll-utils";
import { createAutoResetCounter } from "../../../../hooks/auto-reset-counter";

import { registerCustomElement as registerCustomElementForScratchOff } from "./ScratchOff";
import {
  registerCustomElementForRoWidgetDicexp,
  registerCustomElementForRoWidgetRefLink,
} from "@rotext/solid-components/internal";
import { registerCustomElementForStepRepresentations } from "@dicexp/solid-components";

import EvaluatingWorker from "../../../../workers/dicexp-evaluator?worker";

const CONTENT_ROOT_CLASS = "previewer-content-root";
const PROSE_CLASS = "tuan-prose";

registerCustomElementForRoWidgetRefLink();
registerCustomElementForStepRepresentations("steps-representation");
registerCustomElementForRoWidgetDicexp("dicexp-preview", {
  EvaluatingWorker,
  ErrorAlert,
  Loading,
  tagNameForStepsRepresentation: "steps-representation",
});
registerCustomElementForScratchOff();

const Preview: Component<
  {
    store: EditorStore;
    class?: string;
    setParsingTimeText: Setter<string>;
    onThrowInParsing: (thrown: unknown) => void;
  }
> = (props) => {
  let scrollContainerEl!: HTMLDivElement;
  let widgetAnchorEl!: HTMLDivElement;
  let outputContainerEl!: HTMLDivElement;

  const [err, setErr] = createSignal<Error | null>(null);

  const [scrollHandler, setScrollHandler] = createSignal<(ev: Event) => void>();

  const [highlightElement, setHighlightElement] = createSignal<JSX.Element>();

  onMount(() => {
    const [lookupList, setLookupListRaw] = createLookupList({
      scrollContainer: scrollContainerEl,
      outputContainer: outputContainerEl,
    });

    //==== 文档渲染 ====
    createRendering({
      text: () => props.store.text,
      setLookupListRaw,
      setParsingTimeText: (v) => props.setParsingTimeText(v),
      setErr,
    }, {
      outputContainer: outputContainerEl,
    });

    //==== 滚动同步 ====
    const { handleScroll } = createScrollSyncing({
      text: () => props.store.text,
      topLine: () => props.store.topLine,
      setTopLine: (v) => props.store.topLine = v,
      lookupList,
    }, {
      scrollContainer: scrollContainerEl,
      outputContainer: outputContainerEl,
    });
    setScrollHandler(() => debounceEventHandler(handleScroll));

    //==== 活动行对应元素高亮 ====
    createHighlight({
      activeLines: () => props.store.activeLines,
      lookupList: lookupList,
      setHighlightElement,
    });

    //==== 注册进全局存储 ====
    // NOTE: 目前 scrollContainerEl 就是 previewer 的元素
    registerRoWidgetOwner(scrollContainerEl, {
      proseClass: PROSE_CLASS,
      widgetAnchorElement: () => widgetAnchorEl,
      layoutChange: lookupList,
      level: 1,
    });
  });

  //==== 组件 ====
  return (
    <div
      class={`previewer relative tuan-background overflow-y-auto ${
        props.class ?? ""
      }`}
      ref={scrollContainerEl}
      onScroll={(ev) => scrollHandler()!(ev)}
    >
      <Show when={err()}>
        {(err) => <ErrorAlert error={err()} showsStack={true} />}
      </Show>

      {/* highlight anchor */}
      <div class="relative">{highlightElement()}</div>

      <div class="preview-widget-anchor relative z-10" ref={widgetAnchorEl} />

      <div
        class={"" +
          "relative " + // 作为计算元素高度位移的锚点
          "self-center mx-auto " + // 保持居中，以及撑起父元素
          "break-all " + // 内容的外观样式
          `${PROSE_CLASS} ` +
          ""}
      >
        <div ref={outputContainerEl} />
      </div>
    </div>
  );
};
export default Preview;

function createLookupList(
  els: { scrollContainer: HTMLElement; outputContainer: HTMLElement },
) {
  const [lookupListRaw, setLookupListRaw] = createSignal<LookupList>([]);
  const [lookupList, setLookupList] = createSignal<LookupList>([]);
  function roastLookupList() {
    const _raw = lookupListRaw();
    if (!_raw) return;
    setLookupList(ScrollUtils.roastLookupList(_raw, CONTENT_ROOT_CLASS));
  }
  createEffect(on([lookupListRaw], roastLookupList));
  onMount(() => {
    const observer = new ResizeObserver(roastLookupList);
    observer.observe(els.scrollContainer);
    observer.observe(els.outputContainer);
  });

  return [lookupList, setLookupListRaw] as const;
}

/**
 * 用于处理文档渲染。
 */
function createRendering(
  props: {
    text: () => string;
    setLookupListRaw: (v: LookupListRaw) => void;
    setParsingTimeText(v: string): void;
    setErr: (v: Error | null) => void;
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
      [classModule, styleModule, propsModule, locationModule],
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

      const classMap: Classes = { "relative": true };
      classMap[CONTENT_ROOT_CLASS] = true;
      const vNode = h("article", { class: classMap }, vChildren);

      patch(lastNode, vNode);
      lastNode = vNode;
    } catch (e) {
      if (!(e instanceof Error)) {
        e = new Error(`${e}`);
      }
      props.setErr(e as Error);
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
      if (vnode.data?.location) {
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
  const [pendingAutoScrolls, setPendingAutoScrolls] = createAutoResetCounter();

  {
    const [wasAtBottom, setWasAtBottom] = createSignal<boolean>(false);
    setWasAtBottom(ScrollUtils.isAtBottom(els.scrollContainer));
    (new ResizeObserver(() =>
      setWasAtBottom(ScrollUtils.isAtBottom(els.scrollContainer))
    )).observe(els.outputContainer);

    function scrollToTopLine(opts?: { triggeredBy?: "text-changes" }) {
      const topLineData = props.topLine();
      if (!topLineData.setFrom || topLineData.setFrom === "preview") {
        return;
      }

      const _lookupList = props.lookupList();
      if (!_lookupList.length) return;

      if (!Number.isFinite(topLineData.number)) { // 编辑器要求预览滚动到最底部
        const maxTop = els.scrollContainer.scrollHeight -
          els.scrollContainer.offsetHeight;
        if (maxTop < 0) return;

        const scrollLocal = ScrollUtils.getScrollLocalByY(
          _lookupList,
          maxTop,
          els.outputContainer.offsetHeight,
        );
        const line = ScrollUtils.scrollLocalToLine(scrollLocal, _lookupList);
        props.setTopLine({ number: line, setFrom: "editor" });
        return;
      }

      const scrollResult = ScrollUtils.scrollToLine(
        topLineData.number,
        _lookupList,
        els.scrollContainer,
        wasAtBottom,
        setWasAtBottom,
        { triggeredBy: opts?.triggeredBy },
      );
      if (scrollResult === "scrolled") {
        setPendingAutoScrolls.increase();
      }
    }

    // NOTE: 即使与之前相同也要处理，以在编辑器滚动到预览无法继续同步滚动的位置之下时，
    //       在使预览的高度增加而导致可以继续滚动的位置向下延伸时，预览可以同步位置。
    createEffect(on([props.topLine], () => scrollToTopLine()));
    createEffect(
      on([props.text], () => scrollToTopLine({ triggeredBy: "text-changes" })),
    );
  }

  /**
   * @param scrollContainerEl 滚动内容的容器元素。
   *  除了预览内容之外，还包含可能存在的错误展示等内容。
   */
  function handleScroll(_ev: Event) {
    if (pendingAutoScrolls() > 0) {
      setPendingAutoScrolls.decrease();
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
      const lastTopLine = props.topLine();
      if (
        ScrollUtils.isAtBottom(els.scrollContainer) &&
        lastTopLine.setFrom === "editor" &&
        line <= lastTopLine.number
      ) {
        return;
      }
    }

    props.setTopLine({ number: line, setFrom: "preview" });
  }

  return { handleScroll };
}

function createHighlight(
  props: {
    activeLines: () => ActiveLines | null;
    lookupList: () => LookupList;
    setHighlightElement: (v: () => JSX.Element) => void;
  },
) {
  const [topPx, setTopPx] = createSignal<number>();
  const [heightPx, setHeightPx] = createSignal<number>();

  const el = () => (
    <Show when={props.activeLines()}>
      <div
        class="absolute w-full"
        style={{
          "background-color": "rgb(255, 255, 0, 5%)",
          top: `${topPx()}px`,
          height: `${heightPx()}px`,
        }}
      />
    </Show>
  );

  createEffect(on([props.activeLines, props.lookupList], () => {
    const lookupList_ = props.lookupList();
    if (!lookupList_?.length) return;
    const activeLines_ = props.activeLines();
    if (!activeLines_) return;

    const topLineIndex =
      ScrollUtils.getScrollLocalByLine(lookupList_, activeLines_[0])
        .indexInLookupList;
    const bottomLineIndex = activeLines_[0] === activeLines_[1]
      ? topLineIndex
      : ScrollUtils.getScrollLocalByLine(lookupList_, activeLines_[1])
        .indexInLookupList;

    const topLineItem = lookupList_[topLineIndex]!;
    const bottomLineItem = topLineIndex === bottomLineIndex
      ? topLineItem
      : lookupList_[bottomLineIndex]!;

    setTopPx(topLineItem.offsetTop);
    setHeightPx(
      bottomLineItem.offsetTop + bottomLineItem.element.offsetHeight -
        topLineItem.offsetTop,
    );
  }));

  props.setHighlightElement(el);
}
