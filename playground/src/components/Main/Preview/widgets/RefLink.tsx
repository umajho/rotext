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
import { customElement } from "solid-element";
import { Portal } from "solid-js/web";

import { BsPinFill } from "solid-icons/bs";

import { getPreviewer } from "../../../../stores/previewer";

type DisplayMode = "closed" | "floating" | "pinned";

const LEAVING_DELAY_MS = 100;

const COLLAPSE_HEIGHT_PX = getSizeInPx("6rem");

interface ElementSize {
  widthPx: number;
  heightPx: number;
}

const RefLink: Component<{ address: string }> = (props) => {
  let primeEl: HTMLSpanElement;
  let widgetEl: HTMLDivElement;
  let widgetContainerEl: HTMLDivElement;

  const [hostEl, setHostEl] = createSignal<HTMLElement>();
  const previewer = createMemo(() => {
    if (!hostEl()) return;
    const previewerEl = hostEl().closest(".previewer") as HTMLElement;
    return getPreviewer(previewerEl);
  });

  const [widgetPosition, setWidgetPosition] = createSignal<
    { topPx: number; leftPx: number }
  >({ topPx: 0, leftPx: 0 });

  const [widgetSize, setWidgetSize] = createSignal<
    { widthPx: number; heightPx: number }
  >();
  const [widgetContainerSize, setWidgetContainerSize] = createSignal<
    { widthPx: number; heightPx: number }
  >();

  const collapsible = () => widgetSize()?.heightPx > COLLAPSE_HEIGHT_PX;

  const {
    displayMode,
    collapsed,
    enterHandler,
    leaveHandler,
    pinningTogglerTouchEndHandler,
    pinningToggleHandler,
    refLinkClickHandler,
    pin,
  } = createDisplayModeFSM("closed", collapsible);

  const address = createMemo(() => parseAddress(props.address));
  const addressDescription = createMemo(() => describeAddress(address()));

  onMount(() => {
    setHostEl(primeEl.getRootNode()["host"] as HTMLElement);

    const closestContainerEl = closestContainer(hostEl());
    createEffect(on([previewer().lookupList], () => {
      setWidgetPosition(
        calculateWidgetPosition({
          prime: primeEl,
          widgetAnchor: previewer().widgetAnchorElement(),
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
      size: widgetContainerSize,
      setSize: setWidgetContainerSize,
      removed: () => displayMode() === "closed",
    }, () => widgetContainerEl);

    if (previewer().level === 1) {
      pin(true);
    }
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
        onClick={refLinkClickHandler}
      >
        {">>"}
        {props.address}
      </span>
      <Show when={displayMode() === "pinned"}>
        <div
          style={{
            width: `${widgetContainerSize()?.widthPx}px`,
            height: `${widgetContainerSize()?.heightPx}px`,
          }}
        >
        </div>
      </Show>
      <Portal mount={previewer()?.widgetAnchorElement()}>
        <Show when={displayMode() !== "closed"}>
          <div
            class="absolute border border-white previewer-background"
            ref={widgetContainerEl}
            style={{
              top: `${widgetPosition().topPx}px`,
              left: `${widgetPosition().leftPx}px`,
              "z-index": `-${widgetPosition().topPx | 0}`,
              ...(collapsed()
                ? { "overflow-y": "hidden", height: `${COLLAPSE_HEIGHT_PX}px` }
                : {}),
            }}
            onMouseEnter={enterHandler}
            onMouseLeave={leaveHandler}
          >
            <div ref={widgetEl}>
              <div class="flex flex-col">
                <div class="flex justify-between items-center px-2">
                  <BsPinFill
                    class="cursor-pointer select-none"
                    color={displayMode() === "pinned"
                      ? "red"
                      : /* gray-500 */ "rgb(107 114 128)"}
                    style={displayMode() === "pinned"
                      ? null
                      : { transform: "rotate(45deg)" }}
                    onTouchEnd={pinningTogglerTouchEndHandler}
                    onClick={pinningToggleHandler}
                  />
                  <div class="w-12" />
                  <div>{props.address}</div>
                </div>
                <hr />
                <div class="p-4">
                  {addressDescription()}
                </div>
              </div>
            </div>
          </div>
        </Show>
      </Portal>
    </>
  );
};
export default RefLink;

export function registerCustomElement(tag = "ref-link") {
  customElement(tag, { address: null }, RefLink);
}

function closestContainer(el: HTMLElement): HTMLElement | null {
  do {
    const display = getComputedStyle(el).display;
    if (["block", "list-item", "table-cell"].indexOf(display) >= 0) return el;
    el = el.parentElement;
  } while (el);
  return null;
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

  return {
    displayMode,
    collapsed: () => collapsible() && collapsed(),

    enterHandler: handleEnter,
    leaveHandler: handleLeave,

    pinningTogglerTouchEndHandler: handleTouchPinningTogglerEnd,
    pinningToggleHandler: handleTogglePinning,

    refLinkClickHandler: handleClickRefLink,

    pin,

    setDisplayMode,
  };
}

type Address =
  | (
    & { prefix: string }
    & (
      | { type: "post_number"; postNumber: number }
      | { type: "thread_id"; threadID: string; floorNumber?: number }
      | {
        type: "thread_id_sub";
        threadID: string;
        subThreadID: string;
        floorNumber?: number;
      }
    )
  )
  | { type: "unknown" };

function parseAddress(address: string): Address {
  const prefixAndContent = /^([A-Z]+)\.(.*)$/.exec(address);
  if (!prefixAndContent) return { type: "unknown" };
  const [_1, prefix, content] = prefixAndContent;

  if (/^\d+$/.test(content)) {
    const postNumber = parseInt(content);
    return { type: "post_number", prefix, postNumber };
  }

  const threadIDAndRest = /^([a-z]+)(?:\.([a-z]+))?(?:#(\d+))?$/.exec(content);
  if (!threadIDAndRest) return { type: "unknown" };
  const [_2, threadID, subThreadID, floorNumberText] = threadIDAndRest;

  return {
    prefix,
    threadID,
    ...(floorNumberText ? { floorNumber: parseInt(floorNumberText) } : {}),
    ...(subThreadID
      ? {
        type: "thread_id_sub",
        subThreadID,
      }
      : {
        type: "thread_id",
      }),
  };
}

function describeAddress(address: Address): JSX.Element {
  const dl = ((): JSX.Element => {
    if (address.type === "post_number") {
      return (
        <ul>
          <li>帖号是“{address.postNumber}”的帖子。</li>
        </ul>
      );
    } else if (
      address.type === "thread_id" || address.type === "thread_id_sub"
    ) {
      return (
        <ul>
          <li>
            串号是“{address.threadID}”的串
            {(address.type === "thread_id_sub" ||
                  address.floorNumber !== undefined) && "的，" || "。"}
          </li>
          {address.type === "thread_id_sub" && (
            <li>
              ID是“{address.subThreadID}”的子级串
              {address.floorNumber !== undefined && "的，" || "。"}
            </li>
          )}
          {address.floorNumber !== undefined &&
            (
              <li>
                {address.floorNumber
                  ? `位于第${address.floorNumber}层`
                  : "位于串首"}的帖子。
              </li>
            )}
        </ul>
      );
    } else if (address.type === "unknown") {
      return <p>未知地址</p>;
    }
  })();
  return (
    <div class="prose previewer-prose">
      <p>这里的内容会引用自：</p>
      {dl}
    </div>
  );
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

function getSizeInPx(size: string) {
  const containerEl = document.createElement("div");
  containerEl.style.visibility = "hidden";
  containerEl.style.width = "0";
  containerEl.style.overflow = "hidden";

  const el = document.createElement("div");
  el.style.width = size;

  containerEl.appendChild(el);
  document.body.appendChild(containerEl);

  const sizeInPx = parseFloat(getComputedStyle(el).width);

  containerEl.remove();

  return sizeInPx;
}
