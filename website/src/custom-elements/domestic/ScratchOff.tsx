import { customElement, getCurrentElement } from "solid-element";
import { Component, createSignal, onMount } from "solid-js";

import { adoptStyle } from "@rolludejo/web-internal/shadow-root";

import { styleProvider as styleProviderForPreflight } from "../../styles/preflight";
import { styleProvider as styleProviderForTailwind } from "../../styles/tailwind";

export interface CreateScratchOffComponentOptions {
  innerNoAutoOpenClass: string;
}

function createScratchOffComponent(
  opts: CreateScratchOffComponentOptions & { tagName: string },
): Component {
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
          "bg-white transition-[background-color] duration-[400ms]",
          opts.innerNoAutoOpenClass,
          isRevealed()
            ? "bg-opacity-10"
            : "bg-opacity-100 text-transparent select-none cursor-pointer [&_*]:invisible",
        ].join(" ")}
      >
        <slot name="content" />
      </span>
    );
  };
}

export function registerCustomElement(
  tag: string,
  opts: CreateScratchOffComponentOptions,
) {
  customElement(tag, {}, createScratchOffComponent({ ...opts, tagName: tag }));
}
