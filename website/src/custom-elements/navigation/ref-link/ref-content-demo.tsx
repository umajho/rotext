import { Component, createEffect, createMemo, on, onMount } from "solid-js";
import { render } from "solid-js/web";

import {
  ShadowRootAttacher,
  StyleProvider,
} from "@rolludejo/internal-web-shared/shadow-root";

import { AnkorWidgetNavigationInnerRenderer } from "@rotext/solid-components/internal";

import { createSignalGetterFromWatchable } from "../../hooks";
import { navigateToAddress } from "../../../utils/navigation";
import { Address } from "../../../utils/address";
import { useRotextProcessorsStore } from "../../../contexts/rotext-processors-store";
import { TAG_NAME_MAP } from "../../consts";

import { styleProvider as styleProviderForPreflight } from "../../../styles/preflight";

export function createDemoRefContentRenderer(
  createRendererOpts: { proseClass: string; proseStyleProvider: StyleProvider },
): AnkorWidgetNavigationInnerRenderer {
  const { proseClass, proseStyleProvider } = createRendererOpts;

  return (rawAddrW, rendererOpts) => {
    return {
      isAutoOpenable: true,
      render: (el, renderOpts) => {
        const dispose = render(() => {
          const rawAddr = createSignalGetterFromWatchable(rawAddrW);
          const address = createMemo(() => parseAddress(rawAddr()));
          createEffect(
            on([address], ([address]) =>
              rendererOpts.updateNavigation({
                text: `>>${rawAddr()}`,
                action: () => navigateToAddress(address),
              })),
          );
          return (
            <ShadowRootAttacher
              styleProviders={[styleProviderForPreflight, proseStyleProvider]}
            >
              <AddressDescription address={address()} proseClass={proseClass} />
            </ShadowRootAttacher>
          );
        }, el);
        renderOpts.onCleanup(dispose);
      },
    };
  };
}

const AddressDescription: Component<{
  address: RefAddress;
  proseClass: string;
}> = (props) => {
  let divEl!: HTMLDivElement;

  onMount(() => {
    createEffect(on([() => props.address], ([address]) => {
      const processors = useRotextProcessorsStore()!;
      const processor = processors.currentProvider!();

      const bullets: string[] = [];

      switch (props.address[0]) {
        case "reference/textual": {
          const [_, _prefix, threadID, floorNumber] = //
            address as Extract<RefAddress, { 0: "reference/textual" }>;
          bullets.push(
            `串号是 “${threadID}” 的串` +
              (threadID.includes(".") ? "（子级串）" : "") +
              (floorNumber !== null ? "的，" : "。"),
          );
          if (floorNumber !== null) {
            bullets.push(
              "位于" + (floorNumber ? `第${floorNumber}层` : "串首") +
                "的帖子。",
            );
          }
          break;
        }
        case "reference/numeric": {
          const [_, _prefix, id] = //
            address as Extract<RefAddress, { 0: "reference/numeric" }>;
          bullets.push(`帖号是“${id}”的帖子。`);
          break;
        }
        case "never":
          bullets.push("未知地址。");
          break;
        default:
          props.address satisfies never;
      }

      const src = "这里的内容会引用自：\n\n* " + bullets.join("\n* ");

      const result = processor.process(src, {
        requiresLookupListRaw: false,
        tagNameMap: TAG_NAME_MAP,
      });
      if (result.error) throw new Error("TODO!!");

      divEl.innerHTML = result.html!;
    }));
  });

  return (
    <div
      ref={divEl}
      class={props.proseClass}
      style={{ margin: "1rem" }}
    />
  );
};

type RefAddress = Extract<
  Address,
  { 0: "reference/textual" | "reference/numeric" | "never" }
>;

function parseAddress(address: string): RefAddress {
  const prefixAndContent = /^([A-Z]+)\.(.*)$/.exec(address);
  if (!prefixAndContent) return ["never"];

  const [_1, prefix, content] = //
    prefixAndContent as unknown as [string, string, string];

  if (/^\d+$/.test(content)) {
    const postNumber = parseInt(content);
    return ["reference/numeric", prefix, postNumber];
  }

  const threadIDAndRest = /^([a-z]+)(?:\.([a-z]+))?(?:#(\d+))?$/.exec(content);
  if (!threadIDAndRest) return ["never"];

  const [_2, threadID, subThreadID, floorNumberText] = //
    threadIDAndRest as unknown as [string, string, string?, string?];

  const fullThreadID = threadID + (subThreadID ? ("." + subThreadID) : "");

  return [
    "reference/textual",
    prefix,
    fullThreadID,
    floorNumberText ? parseInt(floorNumberText) : null,
  ];
}
