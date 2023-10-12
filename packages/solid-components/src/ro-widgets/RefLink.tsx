import styles from "./RefLink.module.scss";

import { Component, createMemo, createSignal, JSX } from "solid-js";
import { customElement } from "solid-element";

import {
  getComputedColor,
  getComputedCSSValueOfClass,
  gray500,
  mouseDownNoDoubleClickToSelect,
} from "@rotext/web-utils";

import { createRoWidgetComponent, RoWidgetOwner } from "../ro-widget-core/mod";

import { PinButton, WidgetContainer } from "./support";

const BACKGROUND_COLOR = getComputedColor(
  getComputedCSSValueOfClass("background-color", "tuan-background"),
)!;

interface Properties {
  address: string;
}

const RefLink: Component<Properties> = (outerProps) => {
  const [widgetOwner, setWidgetOwner] = createSignal<RoWidgetOwner>();

  const address = createMemo(() => parseAddress(outerProps.address));
  const addressDescription = createMemo(() =>
    widgetOwner() && describeAddress(address(), widgetOwner()!.proseClass)
  );

  const component = createRoWidgetComponent(
    {
      primeContentComponent: (props) => {
        return (
          <span
            style={{ cursor: props.cursor }}
            onClick={props.onToggleWidget}
            onMouseDown={mouseDownNoDoubleClickToSelect}
          >
            {`>>${outerProps.address}`}
          </span>
        );
      },
      widgetContainerComponent: WidgetContainer,
      widgetContentComponent: (props) => {
        return (
          <div class={styles["ref-link-widget-content"]}>
            <div class={styles["header"]}>
              <PinButton
                displayMode={props.displayMode}
                onClick={props.onClickOnPinIcon}
                onTouchEnd={props.onTouchEndOnPinIcon}
              />
              <div style={{ width: "3rem" }} />
              <div>{outerProps.address}</div>
            </div>
            <hr />
            <div style={{ padding: "1rem" }}>
              {addressDescription()}
            </div>
          </div>
        );
      },
    },
    {
      setWidgetOwner,
      widgetBackgroundColor: () => BACKGROUND_COLOR,
      maskTintColor: () => gray500,
    },
  );

  return <>{component}</>;
};
export default RefLink;

export function registerCustomElement(tag: string) {
  customElement(tag, { address: "" }, RefLink);
}

function parseAddress(address: string): Address {
  const prefixAndContent = /^([A-Z]+)\.(.*)$/.exec(address);
  if (!prefixAndContent) return { type: "unknown" };
  const [_1, prefix, content] = //
    prefixAndContent as unknown as [string, string, string];

  if (/^\d+$/.test(content)) {
    const postNumber = parseInt(content);
    return { type: "post_number", prefix, postNumber };
  }

  const threadIDAndRest = /^([a-z]+)(?:\.([a-z]+))?(?:#(\d+))?$/.exec(content);
  if (!threadIDAndRest) return { type: "unknown" };
  const [_2, threadID, subThreadID, floorNumberText] = //
    threadIDAndRest as unknown as [string, string, string?, string?];

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

function describeAddress(address: Address, proseClass: string): JSX.Element {
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
    } else {
      address.type satisfies "unknown";
      return <p>未知地址</p>;
    }
  })();
  return (
    <div class={proseClass}>
      <p>这里的内容会引用自：</p>
      {dl}
    </div>
  );
}
