import { Component } from "solid-js";

import { BsPinFill } from "solid-icons/bs";

import {
  DisplayMode,
  WidgetContainerProperties,
} from "../../../../hooks/widgets";
import { gray500 } from "../../../../utils/color-consts";
import { computedColorToCSSValue } from "../../../../utils/styles";

export const WidgetContainer: Component<WidgetContainerProperties> = (
  props,
) => {
  return (
    <div
      ref={props.ref}
      class={`border border-white ${props.class}`}
      style={props.style}
      onMouseEnter={props.onMouseEnter}
      onMouseLeave={props.onMouseLeave}
    >
      {props.children}
    </div>
  );
};

export const PinButton: Component<{
  displayMode: () => DisplayMode;
  onTouchEnd: () => void;
  onClick: () => void;
}> = (props) => {
  return (
    <BsPinFill
      class="cursor-pointer select-none"
      color={props.displayMode() === "pinned"
        ? "red"
        : computedColorToCSSValue(gray500)}
      style={props.displayMode() === "pinned"
        ? null
        : { transform: "rotate(45deg)" }}
      onTouchEnd={props.onTouchEnd}
      onClick={props.onClick}
    />
  );
};
