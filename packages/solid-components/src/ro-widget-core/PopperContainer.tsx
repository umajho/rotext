import { Component } from "solid-js";

import { RoWidgetPopperContainerProperties } from "./mod";

const PopperContainer: Component<RoWidgetPopperContainerProperties> = (
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

export default PopperContainer;
