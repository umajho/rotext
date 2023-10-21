import { Component, onMount } from "solid-js";

import { BsPinFill } from "solid-icons/bs";
import { adoptStyle, createStyleProviderFromCSSText } from "@rotext/web-utils";

import { RoWidgetDisplayMode } from "../../ro-widget-core/mod";

import styles from "./PinButton.scss?inline";
import { render } from "solid-js/web";

const styleProvider = createStyleProviderFromCSSText(styles);

const PinButton: Component<{
  displayMode: () => RoWidgetDisplayMode;
  onTouchEnd: () => void;
  onClick: () => void;
}> = (props) => {
  let el!: HTMLDivElement;

  onMount(() => {
    const shadowRoot = el.attachShadow({ mode: "open" });
    adoptStyle(shadowRoot, styleProvider);

    render(() => (
      <BsPinFill
        class={[
          "pin-button",
          props.displayMode() === "pinned" ? "pinned" : "",
        ].join(" ")}
        onTouchEnd={props.onTouchEnd}
        onClick={props.onClick}
      />
    ), shadowRoot);
  });

  return (
    <div
      ref={el}
      style={{ display: "flex" }}
    />
  );
};

export default PinButton;
