import { customElement, getCurrentElement } from "solid-element";
import { Component, createMemo, onMount } from "solid-js";
import { Dynamic } from "solid-js/web";

import {
  OcAlert3,
  OcInfo3,
  OcLightbulb3,
  OcReport3,
  OcStop3,
} from "solid-icons/oc";

import { adoptStyle } from "@rolludejo/internal-web-shared/shadow-root";

import { styleProvider as styleProviderForPreflight } from "../../styles/preflight";
import { styleProvider as styleProviderForTailwind } from "../../styles/tailwind";

function createCalloutComponent(): Component<
  { variant: "note" | "tip" | "important" | "warning" | "caution" }
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

    const color = createMemo(() => {
      switch (props.variant) {
        case "note":
          return "--color-blue-500";
        case "tip":
          return "--color-green-500";
        case "important":
          return "--color-purple-500";
        case "warning":
          return "--color-yellow-500";
        case "caution":
          return "--color-red-500";
      }
    });
    const iconComp = () => {
      switch (props.variant) {
        case "note":
          return OcInfo3;
        case "tip":
          return OcLightbulb3;
        case "important":
          return OcReport3;
        case "warning":
          return OcAlert3;
        case "caution":
          return OcStop3;
      }
    };
    const text = () => {
      switch (props.variant) {
        case "note":
          return "注";
        case "tip":
          return "提示";
        case "important":
          return "重要";
        case "warning":
          return "警告";
        case "caution":
          return "当心";
      }
    };

    return (
      <blockquote
        class="border-l-[6px] py-0.5 md:py-1 px-2 md:px-4 mb-2 md:mb-4 font-normal"
        style={{
          "border-left-color": `var(${color()})`,
          "background-color": `rgba(var(${color()}), .05)`,
        }}
      >
        <div
          class="flex items-center gap-2 mb-2 md:mb-4 py-1"
          style={{ color: `var(${color()})` }}
        >
          <Dynamic component={iconComp()} size={20} />
          <p class="text-lg font-bold">{text()}</p>
        </div>
        <p>
          <slot />
        </p>
      </blockquote>
    );
  };
}

export function registerCustomElement(
  tag: string,
) {
  customElement(
    tag,
    { variant: "note" },
    createCalloutComponent(),
  );
}
