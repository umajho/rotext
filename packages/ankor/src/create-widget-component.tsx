import {
  Component,
  createEffect,
  createMemo,
  createSignal,
  JSX,
  on,
  onCleanup,
  onMount,
  Show,
} from "solid-js";
import { Portal } from "solid-js/web";

import {
  adoptStyle,
  StyleProvider,
} from "@rolludejo/internal-web-shared/shadow-root";

import {
  ComputedColor,
  computedColorToCSSValue,
  getSizeInPx,
} from "@rolludejo/internal-web-shared/styling";

import { createWidgetOwnerAgent, WidgetOwnerAgent } from "./widget-owner-agent";
import { closest, closestContainer, mixColor } from "./utils";
import CollapseMaskLayer from "./CollapseMaskLayer";
import PopperContainer from "./PopperContainer";
import { NO_AUTO_OPEN_CLASS } from "./consts";

const LEAVING_DELAY_MS = 100;

const COLLAPSE_HEIGHT_PX = getSizeInPx("6rem");

export type DisplayMode = "closed" | "floating" | "pinned";

export interface LabelContentComponent {
  cursor: JSX.CSSProperties["cursor"];

  onTogglePopper?: () => void;
}

export interface PopperContainerProperties {
  ref: HTMLDivElement | undefined;

  class?: string;
  style?: JSX.CSSProperties;

  onMouseEnter: () => void;
  onMouseLeave: () => void;

  children: JSX.Element;
}

export interface PopperContentProperties {
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
  LabelContent: Component<LabelContentComponent>;
  PopperContent: Component<PopperContentProperties>;
}, opts: {
  baseStyleProviders?: StyleProvider[];
  openable?: () => boolean;
  autoOpenable?: boolean;
  autoOpenShouldCollapse?: boolean;

  popperBackgroundColor: () => ComputedColor;
  maskTintColor: () => ComputedColor;
}): Component {
  opts.openable ??= () => true;
  opts.autoOpenShouldCollapse ??= true;

  const { LabelContent, PopperContent } = parts;

  let rootEl!: HTMLDivElement;

  // 在执行 handleMount 时必定存在
  let labelEl!: HTMLSpanElement,
    pinAnchorEl: HTMLDivElement;
  // 视情况存在
  let popperContainerEl: HTMLDivElement,
    popperEl: HTMLDivElement;

  let [isCleaningUp, setIsCleaningUp] = createSignal(false);
  onCleanup(() => setIsCleaningUp(true));

  const backgroundColorCSSValue = createMemo(() =>
    computedColorToCSSValue(opts.popperBackgroundColor())
  );

  const [woAgent, setWOAgent] = createSignal<WidgetOwnerAgent>();

  const [popperPosition, setPopperPosition] = //
    createSignal<ElementPosition | null>({ topPx: 0, leftPx: 0 });

  const [canCollapse, setCanCollapse] = createSignal(false);
  const [collapseHeightPx, setCollapseHeightPx] = createSignal(0);

  const maskBaseColor = createMemo((): ComputedColor =>
    mixColor(opts.popperBackgroundColor(), 2 / 3, opts.maskTintColor(), 1 / 3)
  );

  const {
    displayMode,
    collapsed,
    enterHandler,
    leaveHandler,
    pinningTogglerTouchEndHandler,
    pinningToggleHandler,
    labelClickHandler,
    autoOpen,
    expand,
  } = createDisplayModeFSM({
    initialDisplayMode: "closed",
    openable: opts.openable,
    collapsible: canCollapse,
  });

  onMount(() => {
    const shadowRoot = rootEl.getRootNode() as ShadowRoot;

    if (opts.baseStyleProviders) {
      for (const p of opts.baseStyleProviders) {
        adoptStyle(shadowRoot, p);
      }
    }

    const woAgent_ = createWidgetOwnerAgent(shadowRoot.host as HTMLElement);
    setWOAgent(woAgent_);

    const closestContainerEl = closestContainer(labelEl)!;
    const calculateAndSetPopperPosition = () => {
      setPopperPosition(
        calculatePopperPosition({
          label: labelEl,
          popperAnchor: woAgent_.anchorElement,
          closestContainer: closestContainerEl,
        }),
      );
    };
    createEffect(on(
      [() => displayMode() === "floating"],
      ([isFloating]) => {
        woAgent_.layoutChangeObserver
          [isFloating ? "subscribe" : "unsubscribe"](
            calculateAndSetPopperPosition,
          );
        if (isFloating) {
          calculateAndSetPopperPosition();
        }
      },
    ));
    onCleanup(() =>
      woAgent_.layoutChangeObserver.unsubscribe(
        calculateAndSetPopperPosition,
      )
    );

    // 这里是确认 openable 这个 “决定能否打开的函数” 在不在
    if (opts.openable) {
      // 挂件内容的大小，目前只有在需要折叠时才需要侦测（判断是否能折叠）；
      // 挂件容器的大小，目前只有在折叠时才需要侦测（确定遮盖的高度）。

      const { size: popperSize } = createSizeSyncer(
        () => popperEl,
        { removed: () => displayMode() !== "pinned" },
      );
      createEffect(on(
        [popperSize],
        ([size]) => setCanCollapse((size?.heightPx ?? 0) > COLLAPSE_HEIGHT_PX),
      ));
      const { size: popperContainerSize } = createSizeSyncer(
        () => popperContainerEl,
        { removed: () => (displayMode() !== "pinned") || !collapsed() },
      );
      createEffect(on(
        [popperContainerSize],
        ([size]) => setCollapseHeightPx(size?.heightPx ?? 0),
      ));
    }

    if (
      opts.autoOpenable &&
      woAgent_.level === 1 &&
      !closest(labelEl, (el) => el.classList.contains(NO_AUTO_OPEN_CLASS))
    ) {
      autoOpen(!!opts.autoOpenShouldCollapse);
    }

    // 这里是确认 openable 这个 “决定能否打开的函数” 在不在
    if (opts.openable) {
      // 套入 ShadowRootAttacher 后，“直接在 JSX 上视情况切换事件处理器与 undefined” 的
      // 方案对 Dicexp 失效了（但对 RefLink 还有效）。这里通过手动添加/去处来 workaround。
      createEffect(on([opts.openable], ([openable]) => {
        if (openable) {
          labelEl.addEventListener("mouseenter", enterHandler);
          labelEl.addEventListener("mouseleave", leaveHandler);
        } else {
          labelEl.removeEventListener("mouseenter", enterHandler);
          labelEl.removeEventListener("mouseleave", leaveHandler);
        }
      }));
    }
  });

  function handlePortalRef({ shadowRoot }: { shadowRoot: ShadowRoot }) {
    if (opts.baseStyleProviders) {
      for (const p of opts.baseStyleProviders) {
        adoptStyle(shadowRoot, p);
      }
    }
  }

  return () => {
    return (
      <div ref={rootEl} style={{ display: "inline-grid" }}>
        <span ref={labelEl} class="widget-label">
          <LabelContent
            cursor={opts.openable?.()
              ? (displayMode() === "pinned"
                ? (canCollapse()
                  ? (collapsed() ? "zoom-in" : "zoom-out")
                  : undefined)
                : "zoom-in")
              : undefined}
            onTogglePopper={opts.openable?.() ? labelClickHandler : undefined}
          />
        </span>

        <div
          ref={pinAnchorEl}
          style={{ display: displayMode() === "pinned" ? undefined : "none" }}
        />
        <Portal
          ref={handlePortalRef}
          mount={isCleaningUp() || displayMode() === "pinned"
            ? pinAnchorEl
            : woAgent()?.anchorElement}
          useShadow={true}
        >
          <Show when={displayMode() !== "closed"}>
            <PopperContainer
              ref={popperContainerEl}
              style={{
                ...(displayMode() === "pinned"
                  ? {
                    width: "fit-content",
                  }
                  : {
                    position: "absolute",
                    ...(((popperPosition) =>
                      popperPosition
                        ? {
                          top: `${popperPosition.topPx}px`,
                          left: `${popperPosition.leftPx}px`,
                        }
                        : { display: "none" })(popperPosition())),
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
              <div ref={popperEl}>
                <PopperContent
                  displayMode={displayMode}
                  handlerForTouchEndOnPinIcon={pinningTogglerTouchEndHandler}
                  handlerForClickOnPinIcon={pinningToggleHandler}
                />
              </div>
            </PopperContainer>
          </Show>
        </Portal>
      </div>
    );
  };
}

function calculatePopperPosition(
  els: {
    label: HTMLElement;
    popperAnchor: HTMLElement;
    closestContainer: HTMLElement;
  },
): ElementPosition | null {
  if (!els.label.offsetParent) {
    // 为 null 代表在设有 `display: none` 的元素的内部。
    // see: https://stackoverflow.com/a/21696585
    return null;
  }

  const labelRect = els.label.getBoundingClientRect();
  const anchorRect = els.popperAnchor.getBoundingClientRect();
  const closestContainerRect = els.closestContainer.getBoundingClientRect();
  const closestContainerPaddingLeftPx = parseFloat(
    getComputedStyle(els.closestContainer).paddingLeft,
  );

  return {
    topPx: labelRect.bottom - anchorRect.top,
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

  function handleClickLabel() {
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

    labelClickHandler: handleClickLabel,

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
  let resizeObserverForPopper: ResizeObserver | null = null;
  createEffect(on([opts.removed], () => {
    if (!opts.removed()) {
      const el_ = el();

      syncSize(el_);
      resizeObserverForPopper = new ResizeObserver(() => syncSize(el_));
      resizeObserverForPopper.observe(el_);
    } else {
      setSize(null);
      resizeObserverForPopper?.disconnect();
    }
  }));

  return { size };
}
