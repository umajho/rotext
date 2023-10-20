import { Component } from "solid-js";

export { default as PinButton } from "./PinButton";

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
