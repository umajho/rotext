import { Component } from "solid-js";

import { HiSolidChevronDoubleDown } from "solid-icons/hi";

import {
  ComputedColor,
  createStyleProviderFromCSSText,
} from "@rotext/web-utils";

import { ShadowRootAttacher } from "../internal/mod";

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
  const [r, g, b] = props.backgroundColor();
  const baseColorRGB = `${r}, ${g}, ${b}`;
  const topColor = `rgba(${baseColorRGB}, 0)`;
  const bottomColor = `rgb(${baseColorRGB})`;

  return (
    <ShadowRootAttacher styleProviders={[styleProvider]}>
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
    </ShadowRootAttacher>
  );
};

export default CollapseMaskLayer;
