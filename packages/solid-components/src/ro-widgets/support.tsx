import styles from "./support.module.scss";

import { Component } from "solid-js";

import { BsPinFill } from "solid-icons/bs";

import {
  RoWidgetContainerProperties,
  RoWidgetDisplayMode,
} from "../ro-widget-core/mod";

export const WidgetContainer: Component<RoWidgetContainerProperties> = (
  props,
) => {
  return (
    <div
      ref={props.ref}
      class={`${styles["widget-container"]} ${props.class ?? ""}`}
      style={props.style}
      onMouseEnter={props.onMouseEnter}
      onMouseLeave={props.onMouseLeave}
    >
      {props.children}
    </div>
  );
};

export const PinButton: Component<{
  displayMode: () => RoWidgetDisplayMode;
  onTouchEnd: () => void;
  onClick: () => void;
}> = (props) => {
  return (
    <BsPinFill
      class={[
        styles["pin-button"],
        props.displayMode() === "pinned" ? styles["pinned"] : "",
      ].join(" ")}
      onTouchEnd={props.onTouchEnd}
      onClick={props.onClick}
    />
  );
};

export const HorizontalRule: Component = () => (
  <hr
    style={{
      width: "100%",
      height: 0,
      margin: 0,
      color: "inherit",
      "border-style": "solid",
      "border-width": 0,
      "border-top-width": "1px",
    }}
  />
);
