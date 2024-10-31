import { customElement, getCurrentElement } from "solid-element";
import { Component, onMount } from "solid-js";

import { adoptStyle } from "@rolludejo/internal-web-shared/shadow-root";

import { styleProvider as styleProviderForPreflight } from "../../styles/preflight";
import { styleProvider as styleProviderForTailwind } from "../../styles/tailwind";

function createCodeBlockComponent(): Component<{ content: string }> {
  return (props) => {
    const currentElement = getCurrentElement();

    onMount(() => {
      for (const p of [styleProviderForPreflight, styleProviderForTailwind]) {
        adoptStyle(currentElement.shadowRoot!, p);
      }
    });

    return (
      <pre class="bg-[#2d2d2d] text-[#f0c674] p-4 rounded-lg overflow-x-auto font-mono text-sm">
        <code>{props.content}</code>
      </pre>
    );
  };
}

export function registerCustomElement(tag: string) {
  customElement(tag, { content: "" }, createCodeBlockComponent());
}
