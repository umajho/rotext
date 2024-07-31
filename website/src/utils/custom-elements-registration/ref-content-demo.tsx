import { Component, createSignal, JSX, Match, Switch } from "solid-js";
import { render } from "solid-js/web";

import { ShadowRootAttacher, StyleProvider } from "@rolludejo/web-internal";

import {
  RefAddress,
  RefContentRenderer,
} from "@rotext/solid-components/internal";

import { styleProvider as styleProviderForPreflight } from "../../styles/preflight";

export function createDemoRefContentRenderer(
  opts: { proseClass: string; proseStyleProvider: StyleProvider },
): RefContentRenderer {
  return (el, addr, onChange, onCleanup) => {
    const dispose = render(() => {
      const [address, setAddress] = createSignal(addr);
      onChange((addr) => setAddress(addr));
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
    onCleanup(dispose);
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
