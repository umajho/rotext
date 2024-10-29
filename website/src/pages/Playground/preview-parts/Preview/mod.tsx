import {
  Component,
  createEffect,
  createSignal,
  JSX,
  on,
  onMount,
  Show,
} from "solid-js";

// @ts-ignore
import { Idiomorph } from "idiomorph/dist/idiomorph.esm.js";

import * as Ankor from "ankor";

import { ErrorAlert } from "@rotext/solid-components/internal";

import { debounceEventHandler } from "../../../../utils/mod";
import {
  PROSE_CLASS,
  registerCustomElementsOnce,
} from "../../../../utils/custom-elements-registration/mod";

import { ActiveLines, EditorStore, TopLine } from "../../editor-store";
import { createAutoResetCounter } from "../../../../hooks/auto-reset-counter";
import { RotextProcessResult } from "../../../../processors/mod";

import { LookupList, LookupListRaw } from "./internal-types";
import * as ScrollUtils from "./scroll-utils";

registerCustomElementsOnce();

const CONTENT_ROOT_CLASS = "previewer-content-root";

const Preview: Component<
  {
    store: EditorStore;
    processResult: RotextProcessResult;

    class?: string;
    hidden: boolean;
  }
> = (props) => {
  let widgetOwnerEl!: HTMLDivElement;
  let scrollContainerEl!: HTMLDivElement;
  let outputContainerEl!: HTMLDivElement;
  let outputEl!: HTMLElement;

  const [scrollHandler, setScrollHandler] = createSignal<(ev: Event) => void>();

  const [highlightElement, setHighlightElement] = createSignal<JSX.Element>();

  onMount(() => {
    const [lookupList, setLookupListRaw] = createLookupList({
      scrollContainer: scrollContainerEl,
      outputContainer: outputContainerEl,
    });

    { //==== 文档渲染 ====
      createEffect(on([() => props.processResult.html], ([html]) => {
        if (!html) {
          outputEl.innerText = "";
          setLookupListRaw([]);
          return;
        }

        Idiomorph.morph(outputEl, html, { morphStyle: "innerHTML" });

        setLookupListRaw(
          props.processResult.lookupListRawCollector?.(outputEl) ?? [],
        );
      }));
    }

    //==== 滚动同步 ====
    const { handleScroll } = createScrollSyncing({
      inputChangeNotifier: () => props.processResult,
      topLine: () => props.store.topLine,
      setTopLine: (v) => props.store.topLine = v,
      lookupList,
    }, {
      scrollContainer: scrollContainerEl,
      outputContainer: outputContainerEl,
    });
    setScrollHandler(() => debounceEventHandler(handleScroll));

    //==== 活动行对应元素高亮 ====
    setUpHighlight({
      activeLines: () => props.store.activeLines,
      lookupList: lookupList,
      setHighlightElement,
    });

    //==== Widget Owner 相关 ====
    {
      const controller = Ankor.getWidgetOwnerController(widgetOwnerEl)!;

      // 布局改变，通知 widget owner 重新计算挂件位置。
      createEffect(on([lookupList], controller.notifyLayoutChange));
    }
  });

  const { fixScrollOutOfSyncAfterUnhide } = (() => {
    // XXX: 在已经滚动了一段距离时，将预览部分的标签页切到并非 “预览” 的标签页，
    // 再切回 “预览” 标签页，部分浏览器会有不正常的行为。观察如下：
    // - Chrome (126)：预览会莫名其妙往上跳一小段距离，导致触发滚动事件。
    // - Firefox (127): 有概率正常，有概率出现和 Chrome 类似的情况，且即使第一次
    // 情况正常，只是切换标签页不进行其他额外操作，再切换几次还是能触发不正常的
    // 情况。
    // - Safari (16.5) 一切正常。
    //
    // 目前的解决方法就是在取消隐藏后第一次触发滚动事件时终止通常的处理（去让编
    // 辑器部分同步），而是强制让编辑器部分同步原先的滚动位置以恢复同步。

    const [hasScrolled, setHasScrolled] = createSignal(false);
    createEffect(on([() => props.hidden], ([hidden]) => {
      if (!hidden) setHasScrolled(false);
    }));

    function fixScrollOutOfSyncAfterUnhide(): "should_stop" | undefined {
      if (!hasScrolled()) {
        setHasScrolled(true);
        props.store.triggerTopLineUpdateForcedly();
        return "should_stop";
      }
      return undefined;
    }

    return { fixScrollOutOfSyncAfterUnhide };
  })();

  const widgetOwnerData = JSON.stringify(
    {
      level: 1,
    } satisfies Ankor.WidgetOwnerRaw,
  );

  //==== 组件 ====
  return (
    <div
      ref={widgetOwnerEl}
      class={`${Ankor.WIDGET_OWNER_CLASS} contents`}
      data-ankor-widget-owner={widgetOwnerData}
    >
      <div
        class={[
          "w-full px-4",
          ...(props.processResult.error ? [] : ["hidden"]),
        ].join(" ")}
      >
        {
          /*
            temporary workaround for: “Attempting to access a stale value from
            <Show> that could possibly be undefined.”
            FIXME!
          */
        }
        <ErrorAlert
          message={props.processResult.error?.message}
          stack={props.processResult.error?.stack}
        />
      </div>

      <div
        ref={scrollContainerEl}
        class={[
          `${props.hidden ? "hidden" : ""}`,
          "previewer",
          "relative tuan-background overflow-y-auto",
          props.class,
        ].join(" ")}
        onScroll={(ev) => {
          if (fixScrollOutOfSyncAfterUnhide() === "should_stop") return;
          scrollHandler()!(ev);
        }}
      >
        <div class={`${Ankor.ANCHOR_CLASS} relative z-10`} />

        {/* highlight anchor */}
        <div class="relative">{highlightElement()}</div>

        <div
          class={[
            Ankor.CONTENT_CLASS,
            "relative", // 作为计算元素高度位移的锚点
            "self-center mx-auto", // 保持居中，以及撑起父元素
            "break-all", // 内容的外观样式
            PROSE_CLASS,
          ].join(" ")}
        >
          <div ref={outputContainerEl}>
            <article ref={outputEl} class={`relative ${CONTENT_ROOT_CLASS}`} />
          </div>
        </div>
      </div>
    </div>
  );
};
export default Preview;

function createLookupList(
  els: { scrollContainer: HTMLElement; outputContainer: HTMLElement },
) {
  const [lookupListRaw, setLookupListRaw] = createSignal<LookupListRaw>([]);
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
 * 用于处理滚动同步。
 */
function createScrollSyncing(
  props: {
    inputChangeNotifier: () => void;
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
      on(
        [props.inputChangeNotifier],
        () => scrollToTopLine({ triggeredBy: "text-changes" }),
      ),
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

function setUpHighlight(
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
