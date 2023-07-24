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

const RefLink: Component<{ address: string }> = (props) => {
  let primeEl: HTMLSpanElement;
  let widgetEl: HTMLDivElement;

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

  const {
    displayMode,
    enterHandler,
    leaveHandler,
    pinningTogglerTouchEndHandler,
    pinningToggleHandler,
    setDisplayMode,
  } = createDisplayModeFSM("closed");

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

    function syncWidgetSize() {
      const rect = widgetEl.getBoundingClientRect();
      setWidgetSize({ widthPx: rect.width, heightPx: rect.height });
    }
    let resizeObserverForWidget: ResizeObserver | null = null;
    createEffect(() => {
      if (displayMode() !== "closed") {
        syncWidgetSize();
        resizeObserverForWidget = new ResizeObserver(syncWidgetSize);
        resizeObserverForWidget.observe(widgetEl);
      } else {
        resizeObserverForWidget?.disconnect();
      }
    });

    if (previewer().level === 1) {
      setDisplayMode("pinned");
    }
  });

  return (
    <>
      <span
        ref={primeEl}
        style={{ "cursor": displayMode() === "pinned" ? null : "zoom-in" }}
        onMouseEnter={enterHandler}
        onMouseLeave={leaveHandler}
        onClick={() => setDisplayMode("pinned")}
      >
        {">>"}
        {props.address}
      </span>
      <Show when={displayMode() === "pinned"}>
        <div
          style={{
            width: `${widgetSize()?.widthPx}px`,
            height: `${widgetSize()?.heightPx}px`,
          }}
        >
        </div>
      </Show>
      <Portal mount={previewer()?.widgetAnchorElement()}>
        <Show when={displayMode() !== "closed"}>
          <div
            class="absolute border border-white previewer-background"
            ref={widgetEl}
            style={{
              top: `${widgetPosition().topPx}px`,
              left: `${widgetPosition().leftPx}px`,
              "z-index": `-${widgetPosition().topPx | 0}`,
            }}
            onMouseEnter={enterHandler}
            onMouseLeave={leaveHandler}
          >
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
  initialValue: DisplayMode,
) {
  const [displayMode, setDisplayMode] = createSignal(initialValue);

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

  return {
    displayMode,

    enterHandler: handleEnter,
    leaveHandler: handleLeave,

    pinningTogglerTouchEndHandler: handleTouchPinningTogglerEnd,
    pinningToggleHandler: handleTogglePinning,

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
