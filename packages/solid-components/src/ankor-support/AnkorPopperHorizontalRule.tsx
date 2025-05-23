import { Component, JSX } from "solid-js";

export const AnkorPopperHorizontalRule: Component<
  { color: JSX.CSSProperties["color"] }
> = (props) => (
  <hr
    style={{
      width: "100%",
      height: 0,
      margin: 0,
      "border-top-color": props.color,
      "border-style": "solid",
      "border-width": 0,
      "border-top-width": "1px",
    }}
  />
);
