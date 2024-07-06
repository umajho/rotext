import { customElement, getCurrentElement } from "solid-element";
import { Component, createEffect, createSignal, on, onMount } from "solid-js";

import { adoptStyle } from "@rolludejo/web-internal";

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

    onMount(() => {
      for (
        const p of [styleProviderForPreflight, styleProviderForTailwind]
      ) {
        adoptStyle(currentElement.shadowRoot!, p);
      }

      const contentEl = (currentElement.shadowRoot!
        .querySelector('slot[name="content"]')! as HTMLSlotElement)
        .assignedElements()[0]!;
      contentEl.classList.add("[&_*]:invisible");
      createEffect(on([isRevealed], ([isRevealed]) => {
        if (isRevealed) {
          contentEl.classList.remove("[&_*]:invisible");
        }
      }));
    });

    currentElement.addEventListener("click", () => {
      setIsRevealed(true);
    });

    return (
      <span
        class={[
          "bg-white transition-[background-color] duration-[400ms]",
          opts.innerNoAutoOpenClass,
          isRevealed()
            ? "bg-opacity-10"
            : "bg-opacity-100 text-transparent select-none cursor-pointer",
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
