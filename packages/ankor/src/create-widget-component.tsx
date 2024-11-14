import {
  Accessor,
  batch,
  Component,
  createEffect,
  createMemo,
  createSignal,
  JSX,
  on,
  onCleanup,
  onMount,
  Show,
  untrack,
} from "solid-js";
import { Portal } from "solid-js/web";

import { findClosestElementEx } from "@rolludejo/internal-web-shared/dom";
import {
  adoptStyle,
  ShadowRootAttacher,
  StyleProvider,
} from "@rolludejo/internal-web-shared/shadow-root";

import {
  ComputedColor,
  computedColorToCSSValue,
  getSizeInPx,
} from "@rolludejo/internal-web-shared/styling";

import { createWidgetOwnerAgent, WidgetOwnerAgent } from "./widget-owner-agent";
import { mixColor } from "./utils";
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

export interface PopperContentProperties {
  displayMode: () => DisplayMode;
  /**
   * XXX: 只有在挂载后（执行 `onMount` 起）才被定义（不为 `undefined`）。
   */
  widgetOwnerAgentGetter: () => WidgetOwnerAgent | undefined;

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
  let pinAnchorEl!: HTMLDivElement;

  // 在执行 handleMount 时必定存在
  let labelEl!: HTMLSpanElement;
  // 视情况存在
  let popperContainerEl: HTMLDivElement,
    popperEl: HTMLDivElement;

  const backgroundColorCSSValue = createMemo(() =>
    computedColorToCSSValue(opts.popperBackgroundColor())
  );

  const [woAgent, setWOAgent] = createSignal<WidgetOwnerAgent>();

  const [popperPosition, setPopperPosition] = //
    createSignal<ElementPosition | null>({ topPx: 0, leftPx: 0 });

  const [canCollapse, setCanCollapse] = createSignal(false);

  const maskBaseColor = createMemo((): ComputedColor =>
    mixColor(opts.popperBackgroundColor(), 2 / 3, opts.maskTintColor(), 1 / 3)
  );

  const dmFSM = createDisplayModeFSM({
    initialDisplayMode: "closed",
    openable: opts.openable,
    collapsible: canCollapse,
  });

  const [popperWidthPx, setPopperWidthPx] = createSignal<number | null>(null);
  const [popperHeightPx, setPopperHeightPx] = createSignal<number | null>(null);

  onMount(() => {
    const shadowRoot = rootEl.getRootNode() as ShadowRoot;
    const woAgent = createWidgetOwnerAgent(shadowRoot.host as HTMLElement);
    setWOAgent(woAgent);

    { //==== 采纳样式 ====
      if (opts.baseStyleProviders) {
        for (const p of opts.baseStyleProviders) {
          adoptStyle(shadowRoot, p);
        }
      }
    }

    { //==== 持续计算悬浮时悬浮框位置 ====
      function updatePopperPosition() {
        const popperWidthPxOnce = untrack(popperWidthPx);
        if (popperWidthPxOnce !== null) {
          // 代表此时悬浮框还未准备好。将由追踪 signal `popperWidthPx` 的
          // `createEffect` 来处理后续准备好时的情况。

          doUpdatePopperPosition(popperWidthPxOnce);
        }
      }
      createEffect(on([popperWidthPx], ([popperWidthPx]) => {
        if (dmFSM.displayMode() !== "floating") return;
        if (popperWidthPx !== null) {
          doUpdatePopperPosition(popperWidthPx);
        }
      }));
      function doUpdatePopperPosition(popperWidthPx: number) {
        setPopperPosition(
          calculatePopperPosition({
            label: labelEl,
            popperAnchor: woAgent.anchorElement,
          }, { popperWidthPx }),
        );
      }
      createEffect(on(
        [() => dmFSM.displayMode() === "floating"],
        ([isFloating]) => {
          if (isFloating) {
            woAgent.layoutChangeObserver.subscribe(updatePopperPosition);
            updatePopperPosition();
          } else {
            woAgent.layoutChangeObserver.unsubscribe(updatePopperPosition);
            setPopperPosition(null);
          }
        },
      ));
      onCleanup(() =>
        woAgent.layoutChangeObserver.unsubscribe(updatePopperPosition)
      );
    }

    //==== 同步元素大小 ====
    if (opts.openable) { // 确认 openable 这个 “决定能否打开的函数” 在不在。
      // 挂件内容的大小，目前只有在需要折叠时才需要侦测（判断是否能折叠）；
      const { size: popperSize } = createSizeSyncer(
        () => popperEl,
        { enabled: () => dmFSM.displayMode() === "pinned" },
      );
      createEffect(on(
        [popperSize],
        ([size]) => setCanCollapse((size?.heightPx ?? 0) > COLLAPSE_HEIGHT_PX),
      ));
      // 挂件容器的大小，用来：
      // - 确定遮盖的高度；
      // - 确定挂件悬浮时的横向位置。
      const { size: popperContainerSize } = createSizeSyncer(
        () => popperContainerEl,
        { enabled: () => dmFSM.displayMode() !== "closed" },
      );
      createEffect(on(
        [popperContainerSize],
        ([size]) => {
          batch(() => {
            setPopperWidthPx(size?.widthPx ?? null);
            setPopperHeightPx(size?.heightPx ?? null);
          });
        },
      ));
    }

    //==== 自动打开 ====
    if (
      opts.autoOpenable &&
      woAgent.level === 1 &&
      !findClosestElementEx(
        labelEl,
        (el) => el.classList.contains(NO_AUTO_OPEN_CLASS),
      )
    ) {
      dmFSM.autoOpen(!!opts.autoOpenShouldCollapse);
    }

    //==== Workarounds ====
    if (opts.openable) { // 确认 openable 这个 “决定能否打开的函数” 在不在。
      // 套入 ShadowRootAttacher 后，“直接在 JSX 上视情况切换事件处理器与
      // undefined” 的方案对 Dicexp 失效了（但对 RefLink 还有效）。这里通过手动添
      // 加/去处来 workaround。
      createEffect(on([opts.openable], ([openable]) => {
        if (openable) {
          labelEl.addEventListener("mouseenter", dmFSM.handleEnter);
          labelEl.addEventListener("mouseleave", dmFSM.handleLeave);
        } else {
          labelEl.removeEventListener("mouseenter", dmFSM.handleEnter);
          labelEl.removeEventListener("mouseleave", dmFSM.handleLeave);
        }
      }));
    }
  });

  return () => {
    return (
      <div ref={rootEl} style={{ display: "inline-grid" }}>
        <span ref={labelEl} class="widget-label">
          <LabelContent
            cursor={opts.openable?.()
              ? (dmFSM.displayMode() === "pinned"
                ? (canCollapse()
                  ? (dmFSM.collapsed() ? "zoom-in" : "zoom-out")
                  : undefined)
                : "zoom-in")
              : undefined}
            onTogglePopper={opts.openable?.()
              ? dmFSM.handleClickLabel
              : undefined}
          />
        </span>

        <div
          ref={pinAnchorEl}
          style={{
            display: dmFSM.displayMode() === "pinned" ? undefined : "none",
          }}
        />
        <Portal
          mount={dmFSM.displayMode() === "pinned"
            ? pinAnchorEl
            : woAgent()?.anchorElement}
        >
          <ShadowRootAttacher
            styleProviders={opts.baseStyleProviders}
            preventHostStyleInheritance={true}
          >
            <Show when={dmFSM.displayMode() !== "closed"}>
              <PopperContainerEx
                ref={popperContainerEl}
                popperPosition={popperPosition()}
                backgroundColorCSSValue={backgroundColorCSSValue()}
                collapsed={dmFSM.collapsed() ?? false}
                displayMode={dmFSM.displayMode()}
                onMouseEnter={dmFSM.handleEnter}
                onMouseLeave={dmFSM.handleLeave}
              >
                <Show when={dmFSM.collapsed()}>
                  <CollapseMaskLayer
                    containerHeightPx={popperHeightPx()}
                    backgroundColor={maskBaseColor()}
                    onExpand={dmFSM.expand}
                  />
                </Show>
                <div ref={popperEl}>
                  <PopperContent
                    displayMode={dmFSM.displayMode}
                    widgetOwnerAgentGetter={() => untrack(woAgent)}
                    handlerForTouchEndOnPinIcon={dmFSM
                      .handleTouchPinningTogglerEnd}
                    handlerForClickOnPinIcon={dmFSM.handleTogglePinning}
                  />
                </div>
              </PopperContainerEx>
            </Show>
          </ShadowRootAttacher>
        </Portal>
      </div>
    );
  };
}

const PopperContainerEx: Component<{
  ref: HTMLDivElement;

  popperPosition: ElementPosition | null;
  backgroundColorCSSValue: string;
  collapsed: boolean;
  displayMode: DisplayMode;

  onMouseEnter: () => void;
  onMouseLeave: () => void;

  children: JSX.Element;
}> = (props) => {
  const style = createMemo(() => {
    const style: JSX.CSSProperties = {};

    if (props.displayMode === "pinned") {
      style.width = "fit-content";
    } else {
      // 如果改成只在悬浮（且存在 `props.popperPosition`）时进行设置，则在
      // Chrome 及 Firefox 下打开悬浮框时页面会发生位移。Safari 下则一切正常。
      style.position = "absolute";
    }
    if (props.displayMode === "floating" && props.popperPosition) {
      style.transform =
        `translate(${props.popperPosition.leftPx}px,${props.popperPosition.topPx}px)`;
      style["z-index"] = 10;
    }
    style["background-color"] = props.backgroundColorCSSValue;
    if (props.collapsed) {
      style["overflow-y"] = "hidden";
      style.height = `${COLLAPSE_HEIGHT_PX}px`;
    }

    return style;
  });

  return (
    <PopperContainer
      ref={props.ref}
      style={style()}
      onMouseEnter={props.onMouseEnter}
      onMouseLeave={props.onMouseLeave}
    >
      {props.children}
    </PopperContainer>
  );
};

function calculatePopperPosition(
  els: {
    label: HTMLElement;
    popperAnchor: HTMLElement;
  },
  opts: {
    popperWidthPx: number;
  },
): ElementPosition | null {
  if (!els.label.offsetParent) {
    // 为 null 代表在设有 `display: none` 的元素的内部。
    // see: https://stackoverflow.com/a/21696585
    return null;
  }

  const labelRect = els.label.getBoundingClientRect();
  const anchorRect = els.popperAnchor.getBoundingClientRect();

  return {
    topPx: labelRect.bottom - anchorRect.top,
    leftPx: Math.min(
      labelRect.left - anchorRect.left,
      anchorRect.width - opts.popperWidthPx,
    ),
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

    handleEnter,
    handleLeave,

    handleTouchPinningTogglerEnd,
    handleTogglePinning,

    handleClickLabel,

    autoOpen,

    expand,

    setDisplayMode,
  };
}

/**
 * XXX: `el` 并非 reactive，只是由于外部调用此函数时，el 可能作为 `<Show/>` 之内
 * 的 ref，不作为函数的话其值就固定了。（可能是 `on={false}` 时的 `undefined`，也
 * 可能指向先前 `on={true}` 时创建的旧元素。）
 */
function createSizeSyncer(
  elGetter: () => HTMLElement,
  opts: { enabled: Accessor<boolean> },
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
  createEffect(on([opts.enabled], ([enabled]) => {
    const el = elGetter();
    if (enabled) {
      syncSize(el);
      resizeObserverForPopper = new ResizeObserver(() => syncSize(el));
      resizeObserverForPopper.observe(el);
    } else {
      setSize(null);
      resizeObserverForPopper?.disconnect();
    }
  }));

  return { size };
}
