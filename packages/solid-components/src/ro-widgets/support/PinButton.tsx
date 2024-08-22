import { Component } from "solid-js";

import { BsPinFill } from "solid-icons/bs";

import {
  createStyleProviderFromCSSText,
  ShadowRootAttacher,
} from "@rolludejo/web-internal/shadow-root";

import { RoWidgetDisplayMode } from "../../ro-widget-core/mod";

import styles from "./PinButton.scss?inline";

const styleProvider = createStyleProviderFromCSSText(styles);

const PinButton: Component<{
  displayMode: () => RoWidgetDisplayMode;
  onTouchEnd: () => void;
  onClick: () => void;
}> = (props) => {
  return (
    <ShadowRootAttacher
      styleProviders={[styleProvider]}
      hostStyle={{ display: "flex" }}
    >
      <BsPinFill
        class={[
          "pin-button",
          props.displayMode() === "pinned" ? "pinned" : "",
        ].join(" ")}
        onTouchEnd={props.onTouchEnd}
        onClick={props.onClick}
      />
    </ShadowRootAttacher>
  );
};

export default PinButton;
