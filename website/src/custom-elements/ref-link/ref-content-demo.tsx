import { Component, createSignal, JSX, Match, Switch } from "solid-js";
import { render } from "solid-js/web";

import {
  ShadowRootAttacher,
  StyleProvider,
} from "@rolludejo/web-internal/shadow-root";

import { AnkorWidgetNavigationInnerRenderer } from "@rotext/solid-components/internal";

import { styleProvider as styleProviderForPreflight } from "../../styles/preflight";

export function createDemoRefContentRenderer(
  opts: { proseClass: string; proseStyleProvider: StyleProvider },
): AnkorWidgetNavigationInnerRenderer {
  return (el, rawAddr, rOpts) => {
    const dispose = render(() => {
      const [address, setAddress] = createSignal(parseAddress(rawAddr));
      rOpts.onAddressChange((rawAddr) => setAddress(parseAddress(rawAddr)));
      return (
        <ShadowRootAttacher
          styleProviders={[styleProviderForPreflight, opts.proseStyleProvider]}
        >
          <AddressDescription
            address={address()}
            proseClass={opts.proseClass}
          />
        </ShadowRootAttacher>
      );
    }, el);
    rOpts.onCleanup(dispose);
  };
}

const AddressDescription: Component<{
  address: RefAddress;
  proseClass: string;
}> = (props) => {
  return (
    <div class={props.proseClass} style={{ margin: "1rem" }}>
      <p>这里的内容会引用自：</p>
      <AddressDescriptionList address={props.address} />
    </div>
  );
};

const AddressDescriptionList = (
  props: { address: RefAddress },
): JSX.Element => {
  return (
    <Switch>
      <Match when={props.address.type === "post_number"}>
        {(() => {
          const address = props.address as //
          Extract<RefAddress, { type: "post_number" }>;
          return (
            <ul>
              <li>帖号是“{address.postNumber}”的帖子。</li>
            </ul>
          );
        })()}
      </Match>
      <Match
        when={props.address.type === "thread_id" ||
          props.address.type === "thread_id_sub"}
      >
        {(() => {
          const address = props.address as //
          Extract<RefAddress, { type: "thread_id" | "thread_id_sub" }>;
          return (
            <ul>
              <li>
                串号是“{address.threadID}”的串
                {(address.type === "thread_id_sub" ||
                      address.floorNumber !== undefined) && "的，" ||
                  "。"}
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
                    位于{address.floorNumber
                      ? `第${address.floorNumber}层`
                      : "串首"}的帖子。
                  </li>
                )}
            </ul>
          );
        })()}
      </Match>
      <Match when={true}>
        <p>未知地址</p>;
      </Match>
    </Switch>
  );
};

type RefAddress =
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

function parseAddress(address: string): RefAddress {
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
