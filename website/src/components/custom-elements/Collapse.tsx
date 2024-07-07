import { customElement, getCurrentElement } from "solid-element";
import { Component, onMount } from "solid-js";

import { adoptStyle } from "@rolludejo/web-internal";

import { styleProvider as styleProviderForPreflight } from "../../styles/preflight";
import { styleProvider as styleProviderForTailwind } from "../../styles/tailwind";

function createCollapseComponent(): Component {
  return () => {
    const currentElement = getCurrentElement();

    onMount(() => {
      for (
        const p of [styleProviderForPreflight, styleProviderForTailwind]
      ) {
        adoptStyle(currentElement.shadowRoot!, p);
      }
    });

    return (
      <details class="mb-4 p-2 border border-[#444] rounded-lg bg-[#1e1e1e] text-gray-300">
        <summary class="cursor-pointer font-bold text-white px-2 hover:text-blue-500 focus:outline-none">
          <slot name="title">折叠内容</slot>
        </summary>
        <slot name="content" />
      </details>
    );
  };
}

export function registerCustomElement(
  tag: string,
) {
  customElement(tag, {}, createCollapseComponent());
}
