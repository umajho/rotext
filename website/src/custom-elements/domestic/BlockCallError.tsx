import { customElement, getCurrentElement } from "solid-element";
import { Component, onMount } from "solid-js";

import { adoptStyle } from "@rolludejo/internal-web-shared/shadow-root";

import { styleProvider as styleProviderForPreflight } from "../../styles/preflight";
import { styleProvider as styleProviderForTailwind } from "../../styles/tailwind";

function createBlockCallErrorComponent(): Component<{
  "call-type": "transclusion" | "extension" | "";
  "call-name": string;
  "error-type": string;
  "error-value": string | null;
}> {
  return (props) => {
    const currentElement = getCurrentElement();

    const what = () => {
      switch (props["call-type"]) {
        case "transclusion":
          return "嵌入包含";
        case "extension":
          return "扩展";
        default:
          return "未知";
      }
    };

    const message = () => {
      switch (props["error-type"]) {
        case "TODO":
          return "TODO: 实现渲染";
        default:
          return `${props["error-type"]} (${props["error-value"]})`;
      }
    };

    onMount(() => {
      for (const p of [styleProviderForPreflight, styleProviderForTailwind]) {
        adoptStyle(currentElement.shadowRoot!, p);
      }
    });

    return (
      <div class="p-4 my-1 border border-red-500 border-dashed text-red-500">
        <p class="font-bold pb-4">
          {`调用${what()}「`}
          <span class="text-gray-300">{props["call-name"]}</span>
          {`」失败：`}
        </p>
        <p class="text-sm">{message()}</p>
      </div>
    );
  };
}

export function registerCustomElement(tag: string) {
  customElement(
    tag,
    { "call-type": "", "call-name": "", "error-type": "", "error-value": null },
    createBlockCallErrorComponent(),
  );
}
