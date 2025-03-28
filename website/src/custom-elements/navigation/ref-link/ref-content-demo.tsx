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
import { BLOCK_EXTENSION_LIST, TAG_NAME_MAP } from "../../consts";

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

  const processors = useRotextProcessorsStore()!;

  onMount(() => {
    createEffect(
      on(
        [() => props.address, () => processors.currentProvider],
        ([address, currentProcessorProvider]) => {
          if (!currentProcessorProvider) return;
          const processor = currentProcessorProvider();

          const bullets: string[] = [];

          switch (address[0]) {
            case "reference/textualAbsolute": {
              const [_, _prefix, threadID, floorNumber] = address;
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
            case "reference/textualFloorNumber":
              const [_, floorNumber] = address;
              bullets.push("本串的，");
              bullets.push(
                "位于" + (floorNumber ? `第${floorNumber}层` : "串首") +
                  "的帖子。",
              );
              break;
            case "reference/numeric": {
              const [_, _prefix, id] = address;
              bullets.push(`帖号是“${id}”的帖子。`);
              break;
            }
            case "never":
              bullets.push("未知地址。");
              break;
            default:
              address satisfies never;
          }

          const src = "这里的内容会引用自：\n\n* " + bullets.join("\n* ");

          const result = processor.process(src, {
            requiresLookupListRaw: false,
            blockExtensionList: BLOCK_EXTENSION_LIST,
            tagNameMap: TAG_NAME_MAP,
          });
          if (result.error) throw new Error("TODO!!");

          divEl.innerHTML = result.html!;
        },
      ),
    );
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
  {
    0:
      | "reference/textualAbsolute"
      | "reference/textualFloorNumber"
      | "reference/numeric"
      | "never";
  }
>;

function parseAddress(address: string): RefAddress {
  if (address.startsWith("#")) {
    const floorNumberText = address.slice(1);
    if (/^\d+$/.test(floorNumberText)) {
      return ["reference/textualFloorNumber", parseInt(floorNumberText)];
    } else {
      return ["never"];
    }
  }

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
    "reference/textualAbsolute",
    prefix,
    fullThreadID,
    floorNumberText ? parseInt(floorNumberText) : null,
  ];
}
