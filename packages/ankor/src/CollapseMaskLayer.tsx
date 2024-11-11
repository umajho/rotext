import { Component, createMemo, JSX } from "solid-js";

import { HiSolidChevronDoubleDown } from "solid-icons/hi";

import { ShadowRootAttacher } from "@rolludejo/internal-web-shared/shadow-root";

import { ComputedColor } from "@rolludejo/internal-web-shared/styling";

const styles = {
  "collapse-mask-layer": { position: "relative" },
  "pointer-masker": {
    position: "absolute",
    top: 0,
    width: "100%",
    "pointer-events": "none",
    display: "flex",
    "flex-direction": "column",
  },
  "space-taker": {
    flex: "1 1 0",
  },
  "action-area-for-expansion": {
    position: "relative",
    "pointer-events": "auto",
    cursor: "zoom-in",
    height: "2rem", // h-8
  },
  "icon-area": {
    position: "absolute",
    top: "0",
    width: "100%",
    "z-index": "10",
  },
  "aligner": {
    display: "flex",
    "flex-direction": "column",
    "justify-content": "center",
    "align-items": "center",
    height: "2rem", // h-8
  },
  "mask-area": {
    height: "100%",
    "z-index": "0",
  },
} satisfies { [name: string]: JSX.CSSProperties };

const CollapseMaskLayer: Component<
  {
    containerHeightPx: number | null;
    backgroundColor: ComputedColor;
    onExpand: () => void;
  }
> = (
  props,
) => {
  const background = createMemo(() => {
    const { r, g, b } = props.backgroundColor;
    const baseColorRGB = `${r}, ${g}, ${b}`;
    const topColor = `rgba(${baseColorRGB}, 0)`;
    const bottomColor = `rgb(${baseColorRGB})`;
    return `linear-gradient(${topColor}, ${bottomColor})`;
  });

  return (
    <ShadowRootAttacher>
      <div style={styles["collapse-mask-layer"]}>
        <div
          style={{
            ...styles["pointer-masker"],
            ...(props.containerHeightPx &&
              { height: `${props.containerHeightPx}px` }),
          }}
        >
          <div style={styles["space-taker"]} />
          <div
            style={styles["action-area-for-expansion"]}
            onClick={props.onExpand}
          >
            <div style={styles["icon-area"]}>
              <div style={styles["aligner"]}>
                <HiSolidChevronDoubleDown color="white" />
              </div>
            </div>
            <div style={{ ...styles["mask-area"], background: background() }} />
          </div>
        </div>
      </div>
    </ShadowRootAttacher>
  );
};

export default CollapseMaskLayer;
