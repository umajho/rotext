import styles from "./styles.module.scss";

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

import {
  closestContainer,
  ComputedColor,
  computedColorToCSSValue,
  getSizeInPx,
} from "@rotext/web-utils";
import { getCurrentElement, noShadowDOM } from "solid-element";

import { getWidgetOwner, WidgetOwner } from "./widget-owners-store";
import { mixColor } from "./utils";
import CollapseMaskLayer from "./CollapseMaskLayer";

const LEAVING_DELAY_MS = 100;

const COLLAPSE_HEIGHT_PX = getSizeInPx("6rem");

export type DisplayMode = "closed" | "floating" | "pinned";

export interface PrimeContentComponent {
  cursor: JSX.CSSProperties["cursor"];

  onToggleWidget?: () => void;
}

export interface WidgetContainerProperties {
  ref: HTMLDivElement | undefined;

  class?: string;
  style?: JSX.CSSProperties;

  onMouseEnter: () => void;
  onMouseLeave: () => void;

  children: JSX.Element;
}

export interface WidgetContentProperties {
  displayMode: () => DisplayMode;

  handlerForTouchEndOnPinIcon: () => void;
  handlerForClickOnPinIcon: () => void;
}

interface ElementSize {
  widthPx: number;
  heightPx: number;
}

interface ElementPosition {
  topPx: number;
  leftPx: number;
}

export function createWidgetComponent(parts: {
  primeContentComponent: Component<PrimeContentComponent>;
  widgetContainerComponent: Component<WidgetContainerProperties>;
  widgetContentComponent: Component<WidgetContentProperties>;
}, opts: {
  widgetOwnerClass: string;
  innerNoAutoOpenClass?: string;
  setWidgetOwner?: (v: WidgetOwner) => void;
  openable?: () => boolean;
  autoOpenShouldCollapse?: boolean;
  widgetBackgroundColor: () => ComputedColor;
  maskTintColor: () => ComputedColor;
}): Component {
  opts.openable ??= () => true;
  opts.autoOpenShouldCollapse ??= true;

  if (getCurrentElement()) {
    getCurrentElement().innerText = ""; // 清空 fallback
    noShadowDOM();
  }

  let primeEl!: HTMLSpanElement;
  let wContainerEl!: HTMLDivElement; // “w” -> “widget”
  let widgetEl!: HTMLDivElement;

  const backgroundColorCSSValue = createMemo(() =>
    computedColorToCSSValue(opts.widgetBackgroundColor())
  );

  const [widgetOwner, setWidgetOwner] = createSignal<WidgetOwner>();

  const [widgetPosition, setWidgetPosition] = //
    createSignal<ElementPosition | null>({ topPx: 0, leftPx: 0 });

  const [widgetSize, setWidgetSize] = createSignal<ElementSize | null>(null);
  const [wContainerSize, setWContainerSize] = //
    createSignal<ElementSize | null>(null);

  const collapsible = () =>
    opts.openable?.()
      ? (widgetSize()
        ? (widgetSize()?.heightPx ?? 0) > COLLAPSE_HEIGHT_PX
        : null)
      : null;

  const maskBaseColor = createMemo((): ComputedColor =>
    mixColor(opts.widgetBackgroundColor(), 2 / 3, opts.maskTintColor(), 1 / 3)
  );

  const {
    displayMode,
    collapsed,
    enterHandler,
    leaveHandler,
    pinningTogglerTouchEndHandler,
    pinningToggleHandler,
    primeClickHandler,
    autoOpen,
    expand,
  } = createDisplayModeFSM({
    initialDisplayMode: "closed",
    openable: opts.openable,
    collapsible,
  });

  function handleMount() {
    const widgetOwner_ = //
      getWidgetOwner(primeEl.closest("." + opts.widgetOwnerClass)!)!;
    setWidgetOwner(widgetOwner_);
    opts.setWidgetOwner?.(widgetOwner_);

    const closestContainerEl = closestContainer(primeEl)!;
    function calculateAndSetWidgetPosition() {
      setWidgetPosition(
        calculateWidgetPosition({
          prime: primeEl,
          widgetAnchor: widgetOwner_.widgetAnchorElement,
          closestContainer: closestContainerEl,
        }),
      );
    }
    widgetOwner_.onLayoutChange(calculateAndSetWidgetPosition); // TODO: debounce
    calculateAndSetWidgetPosition();

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

    if (
      widgetOwner_.level === 1 &&
      !(opts.innerNoAutoOpenClass &&
        primeEl.closest("." + opts.innerNoAutoOpenClass))
    ) {
      autoOpen(!!opts.autoOpenShouldCollapse);
    }
  }

  return () => {
    onMount(() => {
      handleMount();
    });

    return (
      <>
        <span
          ref={primeEl}
          class={`widget-prime ${styles["widget-prime"]}`}
          onMouseEnter={opts.openable?.() ? enterHandler : undefined}
          onMouseLeave={opts.openable?.() ? leaveHandler : undefined}
        >
          <parts.primeContentComponent
            cursor={opts.openable?.()
              ? (displayMode() === "pinned"
                ? (collapsible()
                  ? (collapsed() ? "zoom-in" : "zoom-out")
                  : undefined)
                : "zoom-in")
              : undefined}
            onToggleWidget={opts.openable?.() ? primeClickHandler : undefined}
          />
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
        <Portal mount={widgetOwner()?.widgetAnchorElement}>
          <Show when={displayMode() !== "closed"}>
            <parts.widgetContainerComponent
              ref={wContainerEl}
              style={{
                position: "absolute",
                ...(((widgetPosition) =>
                  widgetPosition
                    ? {
                      top: `${widgetPosition.topPx}px`,
                      left: `${widgetPosition.leftPx}px`,
                      "z-index": `-${widgetPosition.topPx | 0}`,
                    }
                    : { display: "none" })(widgetPosition())),
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
                  handlerForTouchEndOnPinIcon={pinningTogglerTouchEndHandler}
                  handlerForClickOnPinIcon={pinningToggleHandler}
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
): ElementPosition | null {
  if (!els.prime.offsetParent) {
    // 为 null 代表在设有 `display: none` 的元素的内部。
    // see: https://stackoverflow.com/a/21696585
    return null;
  }

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
  opts: {
    initialDisplayMode: DisplayMode;
    openable: () => boolean;
    collapsible: () => boolean | null;
  },
) {
  const [displayMode, setDisplayMode] = createSignal(opts.initialDisplayMode);
  const [collapsed, setCollapsed] = createSignal(false);

  const [delayedAutoOpen, setDelayedAutoOpen] = createSignal<
    { shouldCollapse: boolean } | null
  >();
  const [userInteracted, setUserInteracted] = createSignal(false);
  createEffect(on([userInteracted], () => {
    if (userInteracted()) {
      setDelayedAutoOpen(null);
    }
  }));

  createEffect(
    on([opts.collapsible, delayedAutoOpen], () => {
      if (!opts.collapsible()) {
        setCollapsed(false);
      }
      if (
        opts.collapsible() === true /* not null */ &&
        delayedAutoOpen()?.shouldCollapse
      ) {
        setCollapsed(true);
      }
    }),
  );
  createEffect(on([opts.openable, delayedAutoOpen], () => {
    if (!opts.openable()) {
      setDisplayMode("closed");
      setUserInteracted(false);
    } else if (delayedAutoOpen()) {
      setDisplayMode("pinned");
    }
  }));

  let leaving = false;
  function handleEnter() {
    if (!opts.openable()) {
      console.warn("should not reach here!");
      return;
    }

    leaving = false;
    if (displayMode() === "closed") {
      setDisplayMode("floating");
    }
  }
  function handleLeave() {
    if (!opts.openable()) {
      console.warn("should not reach here!");
      return;
    }

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
    if (!opts.openable()) {
      console.warn("should not reach here!");
      return;
    }
    setUserInteracted(true);

    setCollapsed(false);
    if (pinningTogglerTouched) {
      setDisplayMode("closed");
    } else {
      const newMode = displayMode() === "pinned" ? "floating" : "pinned";
      setDisplayMode(newMode);
    }
    pinningTogglerTouched = false;
  }

  function handleClickPrime() {
    if (!opts.openable) return;
    setUserInteracted(true);

    if (displayMode() === "pinned") {
      if (!opts.collapsible()) return;
      setCollapsed(!collapsed());
    } else {
      setCollapsed(false);
      setDisplayMode("pinned");
    }
  }

  function autoOpen(shouldCollapse: boolean) {
    setDelayedAutoOpen({ shouldCollapse });
  }

  function expand() {
    if (!opts.openable()) {
      console.warn("should not reach here!");
      return;
    }
    setUserInteracted(true);

    setCollapsed(false);
  }

  return {
    displayMode,
    collapsed: () => opts.collapsible() && collapsed(),

    enterHandler: handleEnter,
    leaveHandler: handleLeave,

    pinningTogglerTouchEndHandler: handleTouchPinningTogglerEnd,
    pinningToggleHandler: handleTogglePinning,

    primeClickHandler: handleClickPrime,

    autoOpen,

    expand,

    setDisplayMode,
  };
}

function createSizeSyncer(
  props: {
    size: () => ElementSize | null;
    setSize: (v: ElementSize | null) => void;
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
      props.setSize(null);
      resizeObserverForWidget?.disconnect();
    }
  }));
}
