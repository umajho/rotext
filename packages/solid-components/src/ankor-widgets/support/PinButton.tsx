import { Component } from "solid-js";

import { BsPinFill } from "solid-icons/bs";

import * as Ankor from "ankor";

import {
  createStyleProviderFromCSSText,
  ShadowRootAttacher,
} from "@rolludejo/internal-web-shared/shadow-root";

import styles from "./PinButton.scss?inline";

const styleProvider = createStyleProviderFromCSSText(styles);

const PinButton: Component<{
  displayMode: () => Ankor.DisplayMode;
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
