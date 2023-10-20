import { Component } from "solid-js";

import { RoWidgetContainerProperties } from "./mod";

const WidgetContainer: Component<RoWidgetContainerProperties> = (
  props,
) => {
  return (
    <div
      ref={props.ref}
      class={props.class ?? ""}
      style={{ border: "1px solid white", ...props.style }}
      onMouseEnter={props.onMouseEnter}
      onMouseLeave={props.onMouseLeave}
    >
      {props.children}
    </div>
  );
};

export default WidgetContainer;
