import styles from "./mod.module.scss";

import { Component } from "solid-js";

import { RoWidgetContainerProperties } from "../../ro-widget-core/mod";

export { default as PinButton } from "./PinButton";

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
