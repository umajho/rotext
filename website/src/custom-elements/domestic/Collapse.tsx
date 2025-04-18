import { customElement, getCurrentElement } from "solid-element";
import { Component, onMount } from "solid-js";

import { adoptStyle } from "@rolludejo/internal-web-shared/shadow-root";

import { styleProvider as styleProviderForPreflight } from "../../styles/preflight";
import { styleProvider as styleProviderForTailwind } from "../../styles/tailwind";

function createCollapseComponent(): Component<
  { title: string; "open-by-default": string }
> {
  return (props) => {
    const currentElement = getCurrentElement();

    onMount(() => {
      for (
        const p of [styleProviderForPreflight, styleProviderForTailwind]
      ) {
        adoptStyle(currentElement.shadowRoot!, p);
      }
    });

    return (
      <details
        class="mb-4 py-2 px-4 border border-[#444] rounded-lg bg-[#1e1e1e] text-gray-300"
        open={!!props["open-by-default"]}
      >
        <summary class="cursor-pointer font-bold text-white hover:text-blue-500 focus:outline-hidden">
          {props.title || "折叠内容"}
        </summary>
        <div class="px-2 pt-2">
          <slot />
        </div>
      </details>
    );
  };
}

export function registerCustomElement(
  tag: string,
) {
  customElement(
    tag,
    { title: "", "open-by-default": "" },
    createCollapseComponent(),
  );
}
