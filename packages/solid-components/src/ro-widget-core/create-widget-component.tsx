import {
  Component,
  createEffect,
  createMemo,
  createSignal,
  JSX,
  on,
  Show,
} from "solid-js";
import { Portal } from "solid-js/web";
import { getCurrentElement, noShadowDOM } from "solid-element";

import {
  adoptStyle,
  ShadowRootAttacher,
  StyleProvider,
} from "@rolludejo/web-internal";

import {
  closestContainer,
  ComputedColor,
  computedColorToCSSValue,
  getSizeInPx,
} from "@rotext/web-utils";

import { getWidgetOwner, WidgetOwner } from "./widget-owners-store";
import { mixColor } from "./utils";
import CollapseMaskLayer from "./CollapseMaskLayer";
import WidgetContainer from "./WidgetContainer";

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
  PrimeContent: Component<PrimeContentComponent>;
  WidgetContent: Component<WidgetContentProperties>;
}, opts: {
  widgetOwnerClass: string;
  innerNoAutoOpenClass?: string;
  setWidgetOwner?: (v: WidgetOwner) => void;
  openable?: () => boolean;
  autoOpenShouldCollapse?: boolean;

  widgetContentStyleProvider?: StyleProvider;
  widgetBackgroundColor: () => ComputedColor;
  maskTintColor: () => ComputedColor;
}): Component {
  opts.openable ??= () => true;
  opts.autoOpenShouldCollapse ??= true;

  const { PrimeContent, WidgetContent } = parts;

  if (getCurrentElement()) {
    getCurrentElement().innerText = ""; // 清空 fallback
    noShadowDOM();
  }

  // 在执行 handleMount 时必定存在
  let primeEl!: HTMLSpanElement,
    widgetFixedAnchorEl: HTMLDivElement;
  // 视情况存在
  let wContainerEl: HTMLDivElement,
    widgetEl: HTMLDivElement; // “w” -> “widget”

  const backgroundColorCSSValue = createMemo(() =>
    computedColorToCSSValue(opts.widgetBackgroundColor())
  );

  const [widgetOwner, setWidgetOwner] = createSignal<WidgetOwner>();

  const [widgetPosition, setWidgetPosition] = //
    createSignal<ElementPosition | null>({ topPx: 0, leftPx: 0 });

  const [canCollapse, setCanCollapse] = createSignal(false);
  const [collapseHeightPx, setCollapseHeightPx] = createSignal(0);

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
    collapsible: canCollapse,
  });

  function handleMount(mntOpts: { host: HTMLElement }) {
    const widgetOwner_ = //
      getWidgetOwner(mntOpts.host.closest("." + opts.widgetOwnerClass)!)!;
    setWidgetOwner(widgetOwner_);
    opts.setWidgetOwner?.(widgetOwner_);

    const closestContainerEl = closestContainer(primeEl)!;
    const calculateAndSetWidgetPosition = () => {
      setWidgetPosition(
        calculateWidgetPosition({
          prime: primeEl,
          widgetAnchor: widgetOwner_.widgetAnchorElement,
          closestContainer: closestContainerEl,
        }),
      );
    };
    createEffect(on(
      [() => displayMode() === "floating"],
      ([isFloating]) => {
        widgetOwner_.layoutChangeObserver
          [isFloating ? "subscribe" : "unsubscribe"](
            calculateAndSetWidgetPosition,
          );
        if (isFloating) {
          calculateAndSetWidgetPosition();
        }
      },
    ));

    // 这里是确认 openable 这个 “决定能否打开的函数” 在不在
    if (opts.openable) {
      // 挂件内容的大小，目前只有在需要折叠时才需要侦测（判断是否能折叠）；
      // 挂件容器的大小，目前只有在折叠时才需要侦测（确定遮盖的高度）。

      const { size: widgetSize } = createSizeSyncer(
        () => widgetEl,
        { removed: () => displayMode() !== "pinned" },
      );
      createEffect(on(
        [widgetSize],
        ([size]) => setCanCollapse((size?.heightPx ?? 0) > COLLAPSE_HEIGHT_PX),
      ));
      const { size: widgetContainerSize } = createSizeSyncer(
        () => wContainerEl,
        { removed: () => (displayMode() !== "pinned") || !collapsed() },
      );
      createEffect(on(
        [widgetContainerSize],
        ([size]) => setCollapseHeightPx(size?.heightPx ?? 0),
      ));
    }

    if (
      widgetOwner_.level === 1 &&
      !(opts.innerNoAutoOpenClass &&
        primeEl.closest("." + opts.innerNoAutoOpenClass))
    ) {
      autoOpen(!!opts.autoOpenShouldCollapse);
    }

    // 这里是确认 openable 这个 “决定能否打开的函数” 在不在
    if (opts.openable) {
      // 套入 ShadowRootAttacher 后，“直接在 JSX 上视情况切换事件处理器与 undefined” 的
      // 方案对 Dicexp 失效了（但对 RefLink 还有效）。这里通过手动添加/去处来 workaround。
      createEffect(on([opts.openable], ([openable]) => {
        if (openable) {
          primeEl.addEventListener("mouseenter", enterHandler);
          primeEl.addEventListener("mouseleave", leaveHandler);
        } else {
          primeEl.removeEventListener("mouseenter", enterHandler);
          primeEl.removeEventListener("mouseleave", leaveHandler);
        }
      }));
    }
  }

  function handlePortalRef({ shadowRoot }: { shadowRoot: ShadowRoot }) {
    if (opts.widgetContentStyleProvider) {
      adoptStyle(shadowRoot, opts.widgetContentStyleProvider);
    }
  }

  return () => {
    return (
      <ShadowRootAttacher
        hostStyle={{ display: "inline" }}
        preventHostStyleInheritance={true}
        onMount={handleMount}
      >
        <span ref={primeEl} class="widget-prime">
          <PrimeContent
            cursor={opts.openable?.()
              ? (displayMode() === "pinned"
                ? (canCollapse()
                  ? (collapsed() ? "zoom-in" : "zoom-out")
                  : undefined)
                : "zoom-in")
              : undefined}
            onToggleWidget={opts.openable?.() ? primeClickHandler : undefined}
          />
        </span>

        <div
          ref={widgetFixedAnchorEl}
          style={{ display: displayMode() === "pinned" ? undefined : "none" }}
        />
        <Portal
          ref={handlePortalRef}
          mount={displayMode() === "pinned"
            ? widgetFixedAnchorEl
            : widgetOwner()?.widgetAnchorElement}
          useShadow={true}
        >
          <Show when={displayMode() !== "closed"}>
            <WidgetContainer
              ref={wContainerEl}
              style={{
                ...(displayMode() === "pinned"
                  ? {
                    width: "fit-content",
                  }
                  : {
                    position: "absolute",
                    ...(((widgetPosition) =>
                      widgetPosition
                        ? {
                          top: `${widgetPosition.topPx}px`,
                          left: `${widgetPosition.leftPx}px`,
                        }
                        : { display: "none" })(widgetPosition())),
                  }),
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
                  containerHeightPx={collapseHeightPx}
                  backgroundColor={maskBaseColor}
                  onExpand={expand}
                />
              </Show>
              <div ref={widgetEl}>
                <WidgetContent
                  displayMode={displayMode}
                  handlerForTouchEndOnPinIcon={pinningTogglerTouchEndHandler}
                  handlerForClickOnPinIcon={pinningToggleHandler}
                />
              </div>
            </WidgetContainer>
          </Show>
        </Portal>
      </ShadowRootAttacher>
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
  el: () => HTMLElement,
  opts: { removed: () => boolean },
) {
  const [size, setSize] = createSignal<ElementSize | null>(null);

  function syncSize(el: HTMLElement) {
    const rect = el.getBoundingClientRect();
    const oldSize = size();
    if (
      oldSize && oldSize.widthPx === rect.width &&
      oldSize.heightPx === rect.height
    ) {
      return;
    }
    setSize({
      widthPx: rect.width,
      heightPx: rect.height,
    });
  }
  let resizeObserverForWidget: ResizeObserver | null = null;
  createEffect(on([opts.removed], () => {
    if (!opts.removed()) {
      const el_ = el();

      syncSize(el_);
      resizeObserverForWidget = new ResizeObserver(() => syncSize(el_));
      resizeObserverForWidget.observe(el_);
    } else {
      setSize(null);
      resizeObserverForWidget?.disconnect();
    }
  }));

  return { size };
}
