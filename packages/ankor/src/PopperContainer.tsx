import { Component } from "solid-js";

import { PopperContainerProperties } from "./create-widget-component";

const PopperContainer: Component<PopperContainerProperties> = (
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
