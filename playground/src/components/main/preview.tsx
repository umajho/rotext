import "./preview.scss";

import {
  Component,
  createEffect,
  createSignal,
  JSX,
  onMount,
  Setter,
  Show,
  untrack,
} from "solid-js";

import { classModule, h, init, Module, styleModule, VNode } from "snabbdom";

import { parse } from "@rotext/parsing";
import { toSnabbdomChildren } from "@rotext/to-html";

const ROOT_CLASS = "rotext-preview-root";

const Preview: Component<
  {
    code: string;
    class?: string;
    setParsingTimeText: Setter<string>;
    onThrowInParsing: (thrown: unknown) => void;
  }
> = (props) => {
  const [err, setErr] = createSignal<Error>(null);
  const [baselineY, setBaselineY] = createSignal(0);
  const [baselineHint, setBaselineHint] = createSignal(null);

  //==== 文档渲染 ====

  let outputEl: HTMLDivElement;
  let patch: ReturnType<typeof init>;
  let lastNode: HTMLElement | VNode;

  const {
    module: locationModule,
    view: locationView,
  } = createLocationModule(ROOT_CLASS);

  onMount(() => {
    patch = init(
      [classModule, styleModule, locationModule],
      undefined,
      { experimental: { fragments: true } },
    );
    lastNode = outputEl;
  });

  createEffect(() => {
    try {
      if (untrack(() => err()) !== null) {
        setErr(null);
      }

      const parsingStart = performance.now();
      const doc = parse(props.code, {
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
  });

  //==== 滚动同步 ====

  function handleScroll(cEl: HTMLElement) { // “cEl” means “container Element”
    if (!locationView.getLookupList().length) return;
    const docEl = (lastNode as VNode).elm as HTMLElement;

    const progress = cEl.scrollTop / (cEl.scrollHeight - cEl.offsetHeight);
    setBaselineY(cEl.scrollHeight * progress);
    if (baselineY() > docEl.offsetHeight) {
      setBaselineY(docEl.offsetHeight);
    } else if (baselineY() < 0) {
      setBaselineY(baselineY);
    }

    const scrollLocal = ScrollSyncUtils.getScrollLocal(
      locationView,
      baselineY(),
    );

    const scrollLocalPreview = (() => {
      const content = scrollLocal.element.textContent;
      if (content.length <= 10) return content;
      return content.slice(0, 10) + "…";
    })();
    const hint = {
      tag: scrollLocal.element.tagName,
      progress: (scrollLocal.progress * 100).toFixed(2) + "%",
      preview: scrollLocalPreview,
    };
    setBaselineHint(`${hint.tag} ${hint.progress} ${hint.preview}`);
  }

  let lastScrollEvent: UIEvent | null = null;
  let handlingScroll = false;
  const handleScrollDebounced: JSX.EventHandlerUnion<HTMLDivElement, UIEvent> =
    (ev) => {
      lastScrollEvent = ev;
      const currentTarget = ev.currentTarget;
      if (handlingScroll) {
        requestAnimationFrame(() => {
          if (lastScrollEvent === ev) {
            handleScroll(currentTarget);
          }
        });
        return;
      }
      handlingScroll = true;
      requestAnimationFrame(() => {
        handleScroll(currentTarget);
        handlingScroll = false;
      });
    };

  //==== 组件 ====
  return (
    <div
      class={`${props.class ?? ""} break-all prose previewer overflow-y-auto`}
      onScroll={handleScrollDebounced}
    >
      <div class="relative h-0 z-50">
        <div class="absolute w-full" style={{ top: `${baselineY()}px` }}>
          <div
            class="flex flex-row gap-2 items-center"
            style="transform: translate(0, -50%)"
          >
            <div class="flex-1 border-red-500 w-full h-0 border-[0.1px]" />
            <div class="text-red-500">{baselineHint()}</div>
            <div class="flex-1 border-red-500 w-full h-0 border-[0.1px]" />
          </div>
        </div>
      </div>
      <Show when={err()}>
        <ErrorAlert error={err()} showsStack={true} />
      </Show>
      <div ref={outputEl} />
    </div>
  );
};
export default Preview;

const ErrorAlert: Component<{
  error: Error;
  showsStack: boolean;
}> = (props) => {
  return (
    <div class="alert alert-error shadow-lg overflow-scroll">
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

interface LocationView {
  getLookupList(): ElementLocationPair[];
}
function createLocationModule(
  rootClass: string,
): { module: Module; view: LocationView } {
  type LookupListRaw = Omit<ElementLocationPair, "offsetTop">[];
  function roastLookupList(raw: LookupListRaw) {
    // 按理来讲应该已经是按起始行数排序的了，不过以免万一就再排序一次。
    // 其实原本还会保证越深的元素排在越后面，不过后面的操作不用考虑这件事。
    raw.sort((a, b) => a.location.start.line - b.location.start.line);

    const [rootElement, rootElementViewportOffsetTop] = (() => {
      if (!raw.length) return [null, null];
      let el = raw[0].element;
      while (!el.classList.contains(rootClass)) {
        el = el.parentElement;
      }
      return [el, el.getBoundingClientRect().top];
    })();

    const roasted = raw as ElementLocationPair[];
    for (const item of roasted) {
      const itemElementViewportOffsetTop =
        item.element.getBoundingClientRect().top;
      item.offsetTop = itemElementViewportOffsetTop -
        rootElementViewportOffsetTop;
    }

    return roasted;
  }

  let loookupListRaw!: LookupListRaw;
  let lookupList!: ElementLocationPair[];

  const module = {
    pre: () => {
      loookupListRaw = [];
      lookupList = [];
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
      lookupList = roastLookupList(loookupListRaw);

      lookupList = lookupList.reduce((acc, cur) => {
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
    },
  };
  return { module, view: { getLookupList: () => lookupList } };
}

/**
 * baseline 所穿过的元素、到达下一个这样的元素的进度，以及这个元素对应于原始输入的行数。
 */
interface ScrollLocal {
  element: HTMLElement;
  progress: number;
  lines: { start: number; end: number };
}
const ScrollSyncUtils = {
  getScrollLocal(
    locationView: LocationView,
    baselineY: number,
  ): ScrollLocal {
    const lookupList = locationView.getLookupList();

    const localIndex = ScrollSyncUtils.binarySearch(lookupList, (item, i) => {
      if (item.offsetTop > baselineY) return "less";

      const nextItem = lookupList[i + 1];
      if (!nextItem || baselineY < nextItem.offsetTop) return true;
      return "greater";
    }) ?? 0;
    const localItem = lookupList[localIndex];
    const nextItem: ElementLocationPair | undefined =
      lookupList[localIndex + 1];

    const offsetTop = localItem.offsetTop;
    const nextOffsetTop = nextItem
      ? nextItem.offsetTop
      : localItem.offsetTop + localItem.element.offsetHeight;

    const progress = (baselineY - offsetTop) / (nextOffsetTop - offsetTop);

    const endLine = nextItem
      ? nextItem.location.start.line - 1
      : localItem.location.end.line;

    return {
      element: localItem.element,
      progress,
      lines: { start: localItem.location.start.line, end: endLine },
    };
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
};
