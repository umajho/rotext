import { Component } from "solid-js";

import { BsPinFill } from "solid-icons/bs";

import * as Ankor from "ankor";

import {
  createStyleProviderFromCSSText,
  ShadowRootAttacher,
} from "@rolludejo/internal-web-shared/shadow-root";

import styles from "./PinButton.css?inline";

const styleProvider = createStyleProviderFromCSSText(styles);

const PinButton: Component<{
  displayMode: () => Ankor.DisplayMode;
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
        onClick={props.onClick}
      />
    </ShadowRootAttacher>
  );
};

export default PinButton;
