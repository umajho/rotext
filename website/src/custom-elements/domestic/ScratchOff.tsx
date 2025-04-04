import { customElement, getCurrentElement } from "solid-element";
import { Component, createSignal, onMount } from "solid-js";

import { NO_AUTO_OPEN_CLASS } from "ankor";

import { adoptStyle } from "@rolludejo/internal-web-shared/shadow-root";

import { styleProvider as styleProviderForPreflight } from "../../styles/preflight";
import { styleProvider as styleProviderForTailwind } from "../../styles/tailwind";

function createScratchOffComponent(): Component {
  return () => {
    const [isRevealed, setIsRevealed] = createSignal(false);

    const currentElement = getCurrentElement();

    let wrapperEl!: HTMLSpanElement;

    onMount(() => {
      for (
        const p of [styleProviderForPreflight, styleProviderForTailwind]
      ) {
        adoptStyle(currentElement.shadowRoot!, p);
      }

      // 不知为何，`span` 上的 `onClick` 不会被触发，只好像这样手动添加事件监听
      // 器。
      function reveal() {
        setIsRevealed(true);
        wrapperEl.removeEventListener("click", reveal);
      }
      wrapperEl.addEventListener("click", reveal);
    });

    return (
      <span
        ref={wrapperEl}
        class={[
          " transition-[background-color] duration-[400ms]",
          NO_AUTO_OPEN_CLASS,
          isRevealed()
            ? "bg-white/10"
            : "bg-white text-transparent select-none cursor-pointer [&_*]:invisible",
        ].join(" ")}
      >
        <slot />
      </span>
    );
  };
}

export function registerCustomElement(tag: string) {
  customElement(tag, {}, createScratchOffComponent());
}
