import {
  Component,
  createEffect,
  createMemo,
  createSignal,
  JSX,
  on,
  onMount,
  Show,
} from "solid-js";
import { Portal } from "solid-js/web";

import { HiSolidChevronDoubleDown } from "solid-icons/hi";

import {
  ComputedColor,
  computedColorToCSSValue,
  getSizeInPx,
} from "../utils/styles";
import { getWidgetOwner } from "../stores/widget-owners";
import { closestContainer } from "../utils/elements";

const LEAVING_DELAY_MS = 100;

const COLLAPSE_HEIGHT_PX = getSizeInPx("6rem");

export type DisplayMode = "closed" | "floating" | "pinned";

export interface WidgetContainerProperties {
  ref: HTMLDivElement | undefined;

  class: string;
  style: JSX.CSSProperties;

  onMouseEnter: () => void;
  onMouseLeave: () => void;

  children: JSX.Element;
}

export interface WidgetContentProperties {
  displayMode: () => DisplayMode;

  onTouchEndOnPinIcon: () => void;
  onClickOnPinIcon: () => void;
}

interface ElementSize {
  widthPx: number;
  heightPx: number;
}

export function createWidgetComponent(parts: {
  primeContentComponent: Component;
  widgetContainerComponent: Component<WidgetContainerProperties>;
  widgetContentComponent: Component<WidgetContentProperties>;
}, opts: {
  widgetBackgroundColor: () => ComputedColor;
  maskTintColor: () => ComputedColor;
}): Component {
  let primeEl: HTMLSpanElement;
  let wContainerEl: HTMLDivElement; // “w” -> “widget”
  let widgetEl: HTMLDivElement;

  const backgroundColorCSSValue = createMemo(() =>
    computedColorToCSSValue(opts.widgetBackgroundColor())
  );

  const [hostEl, setHostEl] = createSignal<HTMLElement>();
  const widgetOwner = createMemo(() => {
    if (!hostEl()) return;
    const previewerEl = hostEl().closest(".previewer") as HTMLElement;
    return getWidgetOwner(previewerEl);
  });

  const [widgetPosition, setWidgetPosition] = createSignal<
    { topPx: number; leftPx: number }
  >({ topPx: 0, leftPx: 0 });

  const [widgetSize, setWidgetSize] = createSignal<ElementSize>();
  const [wContainerSize, setWContainerSize] = createSignal<ElementSize>();

  const collapsible = () => widgetSize()?.heightPx > COLLAPSE_HEIGHT_PX;

  const maskBaseColor = createMemo((): ComputedColor | null =>
    mixColor(opts.widgetBackgroundColor(), 2 / 3, opts.maskTintColor(), 1 / 3)
  );

  const {
    displayMode,
    collapsed,
    enterHandler,
    leaveHandler,
    pinningTogglerTouchEndHandler,
    pinningToggleHandler,
    refLinkClickHandler: primeClickHandler,
    pin,
    expand,
  } = createDisplayModeFSM("closed", collapsible);

  function handleMount() {
    setHostEl(primeEl.getRootNode()["host"] as HTMLElement);

    const closestContainerEl = closestContainer(hostEl());
    createEffect(on([widgetOwner().layoutChange], () => { // TODO: debounce
      setWidgetPosition(
        calculateWidgetPosition({
          prime: primeEl,
          widgetAnchor: widgetOwner().widgetAnchorElement(),
          closestContainer: closestContainerEl,
        }),
      );
    }));

    createSizeSyncer({
      size: widgetSize,
      setSize: setWidgetSize,
      removed: () => displayMode() === "closed",
    }, () => widgetEl);

    createSizeSyncer({
      size: wContainerSize,
      setSize: setWContainerSize,
      removed: () => displayMode() === "closed",
    }, () => wContainerEl);

    if (widgetOwner().level === 1) {
      pin(true);
    }
  }

  return (props) => {
    onMount(() => {
      handleMount();
    });

    return (
      <>
        <span
          ref={primeEl}
          style={{
            "cursor": displayMode() === "pinned"
              ? (collapsible() ? (collapsed() ? "zoom-in" : "zoom-out") : null)
              : "zoom-in",
          }}
          onMouseEnter={enterHandler}
          onMouseLeave={leaveHandler}
          onClick={primeClickHandler}
        >
          <parts.primeContentComponent />
        </span>
        <Show when={displayMode() === "pinned"}>
          <div
            style={{
              width: `${wContainerSize()?.widthPx}px`,
              height: `${wContainerSize()?.heightPx}px`,
            }}
          >
          </div>
        </Show>
        <Portal mount={widgetOwner()?.widgetAnchorElement()}>
          <Show when={displayMode() !== "closed"}>
            <parts.widgetContainerComponent
              ref={wContainerEl}
              class="absolute"
              style={{
                top: `${widgetPosition().topPx}px`,
                left: `${widgetPosition().leftPx}px`,
                "z-index": `-${widgetPosition().topPx | 0}`,
                "background-color": backgroundColorCSSValue(),
                ...(collapsed()
                  ? {
                    "overflow-y": "hidden",
                    height: `${COLLAPSE_HEIGHT_PX}px`,
                  }
                  : {}),
              }}
              onMouseEnter={enterHandler}
              onMouseLeave={leaveHandler}
            >
              <Show when={collapsed()}>
                <CollapseMaskLayer
                  containerHeightPx={() => wContainerSize()?.heightPx}
                  backgroundColor={maskBaseColor}
                  onExpand={expand}
                />
              </Show>
              <div ref={widgetEl}>
                <parts.widgetContentComponent
                  displayMode={displayMode}
                  onTouchEndOnPinIcon={pinningTogglerTouchEndHandler}
                  onClickOnPinIcon={pinningToggleHandler}
                />
              </div>
            </parts.widgetContainerComponent>
          </Show>
        </Portal>
      </>
    );
  };
}

function calculateWidgetPosition(
  els: {
    prime: HTMLElement;
    widgetAnchor: HTMLElement;
    closestContainer: HTMLElement;
  },
) {
  const primeRect = els.prime.getBoundingClientRect();
  const anchorRect = els.widgetAnchor.getBoundingClientRect();
  const closestContainerRect = els.closestContainer.getBoundingClientRect();
  const closestContainerPaddingLeftPx = parseFloat(
    getComputedStyle(els.closestContainer).paddingLeft,
  );

  return {
    topPx: primeRect.bottom - anchorRect.top,
    leftPx: closestContainerRect.left + closestContainerPaddingLeftPx -
      anchorRect.left,
  };
}

function createDisplayModeFSM(
  initialDisplayMode: DisplayMode,
  collapsible: () => boolean,
) {
  const [displayMode, setDisplayMode] = createSignal(initialDisplayMode);
  const [collapsed, setCollapsed] = createSignal(false);

  createEffect(on([collapsible], () => {
    if (!collapsible()) {
      setCollapsed(false);
    }
  }));

  let leaving = false;
  function handleEnter() {
    leaving = false;
    if (displayMode() === "closed") {
      setDisplayMode("floating");
    }
  }
  function handleLeave() {
    if (leaving) return;
    if (displayMode() === "floating") {
      leaving = true;
      setTimeout(() => {
        if (leaving) {
          setDisplayMode("closed");
          leaving = false;
        }
      }, LEAVING_DELAY_MS);
    }
  }

  let pinningTogglerTouched = false;
  function handleTouchPinningTogglerEnd() {
    pinningTogglerTouched = true;
    // 为防止有的浏览器 onClick 发生在 onTouchEnd 之前，
    // 这里也在一定时间后把 `pinIconTouched` 重置一下。
    setTimeout(() => pinningTogglerTouched = false, 100);
  }
  function handleTogglePinning() {
    setCollapsed(false);
    if (pinningTogglerTouched) {
      setDisplayMode("closed");
    } else {
      const newMode = displayMode() === "pinned" ? "floating" : "pinned";
      setDisplayMode(newMode);
    }
    pinningTogglerTouched = false;
  }

  function handleClickRefLink() {
    if (!collapsible()) return;
    if (displayMode() === "pinned") {
      setCollapsed(!collapsed());
    } else {
      setCollapsed(false);
      setDisplayMode("pinned");
    }
  }

  function pin(shouldCollapse: boolean) {
    if (shouldCollapse) {
      // NOTE: 调用本函数的时候，挂件可能还没创建，导致 `collapsible()` 返回假。
      //       这里就不检查 `collapsible()`，直接设置了。这么做也不存在问题。

      setCollapsed(true);
    }
    setDisplayMode("pinned");
  }

  function expand() {
    setCollapsed(false);
  }

  return {
    displayMode,
    collapsed: () => collapsible() && collapsed(),

    enterHandler: handleEnter,
    leaveHandler: handleLeave,

    pinningTogglerTouchEndHandler: handleTouchPinningTogglerEnd,
    pinningToggleHandler: handleTogglePinning,

    refLinkClickHandler: handleClickRefLink,

    pin,

    expand,

    setDisplayMode,
  };
}

function createSizeSyncer(
  props: {
    size: () => ElementSize | null;
    setSize: (v: ElementSize) => void;
    removed: () => boolean;
  },
  el: () => HTMLElement,
) {
  function syncSize(el: HTMLElement) {
    const rect = el.getBoundingClientRect();
    const oldSize = props.size();
    if (
      oldSize && oldSize.widthPx === rect.width &&
      oldSize.heightPx === rect.height
    ) {
      return;
    }
    props.setSize({
      widthPx: rect.width,
      heightPx: rect.height,
    });
  }
  let resizeObserverForWidget: ResizeObserver | null = null;
  createEffect(on([props.removed], () => {
    if (!props.removed()) {
      const el_ = el();

      syncSize(el_);
      resizeObserverForWidget = new ResizeObserver(() => syncSize(el_));
      resizeObserverForWidget.observe(el_);
    } else {
      resizeObserverForWidget?.disconnect();
    }
  }));
}

function mixColor(
  colorA: ComputedColor,
  weightA: number,
  colorB: ComputedColor,
  weightB: number,
) {
  const mixedColor: ComputedColor = [0, 0, 0, null];
  for (let i = 0; i < 3; i++) {
    mixedColor[i] = (colorA[i] * weightA + colorB[i] * weightB) | 0;
  }
  return mixedColor;
}

const CollapseMaskLayer: Component<
  {
    containerHeightPx: () => number;
    backgroundColor: () => ComputedColor;
    onExpand: () => void;
  }
> = (
  props,
) => {
  const [r, g, b] = props.backgroundColor();
  const baseColorRGB = `${r}, ${g}, ${b}`;
  const topColor = `rgba(${baseColorRGB}, 0)`;
  const bottomColor = `rgb(${baseColorRGB})`;

  return (
    <div class="relative">
      <div
        class="absolute top-0 w-full pointer-events-none"
        style={{ height: `${props.containerHeightPx()}px` }}
      >
        <div class="flex flex-col h-full">
          <div class="flex-1" />
          <div
            class="relative pointer-events-auto cursor-zoom-in h-8"
            onClick={props.onExpand}
          >
            <div class="absolute top-0 w-full z-10">
              <div class="flex flex-col justify-center items-center h-8">
                <HiSolidChevronDoubleDown />
              </div>
            </div>
            <div
              class="h-full z-0"
              style={{
                background: `linear-gradient(${topColor}, ${bottomColor})`,
              }}
            />
          </div>
        </div>
      </div>
    </div>
  );
};
