import { Component, onMount } from "solid-js";
import { render } from "solid-js/web";

import { HiSolidChevronDoubleDown } from "solid-icons/hi";

import {
  adoptStyle,
  ComputedColor,
  createStyleProviderFromCSSText,
} from "@rotext/web-utils";

import styles from "./CollapseMaskLayer.scss?inline";

const styleProvider = createStyleProviderFromCSSText(styles);

const CollapseMaskLayer: Component<
  {
    containerHeightPx: () => number | undefined;
    backgroundColor: () => ComputedColor;
    onExpand: () => void;
  }
> = (
  props,
) => {
  let el!: HTMLDivElement;

  const [r, g, b] = props.backgroundColor();
  const baseColorRGB = `${r}, ${g}, ${b}`;
  const topColor = `rgba(${baseColorRGB}, 0)`;
  const bottomColor = `rgb(${baseColorRGB})`;

  onMount(() => {
    const shadowRoot = el.attachShadow({ mode: "open" });
    adoptStyle(shadowRoot, styleProvider);

    render(() => (
      <div class="collapse-mask-layer">
        <div
          class="pointer-masker"
          style={{ height: `${props.containerHeightPx()}px` }}
        >
          <div class="space-taker" />
          <div class="action-area-for-expansion" onClick={props.onExpand}>
            <div class="icon-area">
              <div class="aligner">
                <HiSolidChevronDoubleDown />
              </div>
            </div>
            <div
              class="mask-area"
              style={{
                background: `linear-gradient(${topColor}, ${bottomColor})`,
              }}
            />
          </div>
        </div>
      </div>
    ), shadowRoot);
  });

  return <div ref={el} />;
};

export default CollapseMaskLayer;
