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

type DisplayMode = "hidden" | "floating" | "pinned";

const LEAVING_DELAY_MS = 200;

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

  const [displayMode, setDisplayMode] = createSignal<DisplayMode>("hidden");

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
      if (displayMode() !== "hidden") {
        syncWidgetSize();
        resizeObserverForWidget = new ResizeObserver(syncWidgetSize);
        resizeObserverForWidget.observe(widgetEl);
      } else {
        resizeObserverForWidget?.disconnect();
      }
    });
  });

  const { enterHandler, leaveHandler, pinHandler, pinningToggleHandler } =
    createDisplayModeFSM({ displayMode, setDisplayMode });

  return (
    <>
      <span
        ref={primeEl}
        style={{ "cursor": displayMode() === "pinned" ? null : "zoom-in" }}
        onMouseEnter={enterHandler}
        onMouseLeave={leaveHandler}
        onClick={pinHandler}
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
        <Show when={displayMode() !== "hidden"}>
          <div
            class="absolute border border-white previewer-background"
            ref={widgetEl}
            style={{
              top: `${widgetPosition().topPx}px`,
              left: `${widgetPosition().leftPx}px`,
            }}
            onMouseEnter={enterHandler}
            onMouseLeave={leaveHandler}
          >
            <div class="flex flex-col">
              <div class="flex justify-between items-center px-2">
                <BsPinFill
                  class=" cursor-pointer"
                  color={displayMode() === "pinned"
                    ? "red"
                    : /* gray-500 */ "rgb(107 114 128)"}
                  style={displayMode() === "pinned"
                    ? null
                    : { transform: "rotate(45deg)" }}
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
  props: {
    displayMode: () => DisplayMode;
    setDisplayMode: (v: DisplayMode) => void;
  },
) {
  let leaving = false;
  function handleEnter() {
    leaving = false;
    if (props.displayMode() === "hidden") {
      props.setDisplayMode("floating");
    }
  }
  function handleLeave() {
    if (leaving) return;
    if (props.displayMode() === "floating") {
      leaving = true;
      setTimeout(() => {
        if (leaving) {
          props.setDisplayMode("hidden");
          leaving = false;
        }
      }, LEAVING_DELAY_MS);
    }
  }

  function handlePin() {
    props.setDisplayMode("pinned");
  }
  function handleTogglePinning() {
    const newMode = props.displayMode() === "pinned" ? "floating" : "pinned";
    props.setDisplayMode(newMode);
  }

  return {
    enterHandler: handleEnter,
    leaveHandler: handleLeave,
    pinHandler: handlePin,
    pinningToggleHandler: handleTogglePinning,
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
        <dl>
          <dt>帖子</dt>
          <dd>帖号「{address.postNumber}」</dd>
        </dl>
      );
    } else if (
      address.type === "thread_id" || address.type === "thread_id_sub"
    ) {
      return (
        <dl>
          <dt>串</dt>
          <dd>串号「{address.threadID}」</dd>
          {address.type === "thread_id_sub" && (
            <>
              <dt>子级串</dt>
              <dd>ID「{address.subThreadID}」</dd>
            </>
          )}
          {address.floorNumber !== undefined &&
            (
              <>
                <dt>帖子</dt>
                <dd>
                  {address.floorNumber
                    ? `位于第${address.floorNumber}层`
                    : "位于串首"}
                </dd>
              </>
            )}
        </dl>
      );
    } else if (address.type === "unknown") {
      return <p>未知地址</p>;
    }
  })();
  return (
    <div class="previewer-prose">
      <p>这里的内容会引用自：</p>
      {dl}
    </div>
  );
}
