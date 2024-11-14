import { Component, JSX } from "solid-js";

interface PopperContainerProperties {
  ref: HTMLDivElement | undefined;

  class?: string;
  style?: JSX.CSSProperties;

  onMouseEnter: () => void;
  onMouseLeave: () => void;

  children: JSX.Element;
}

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
