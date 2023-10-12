import { Component } from "solid-js";

import { BsPinFill } from "solid-icons/bs";

import { gray500 } from "@rotext/web-utils";
import { computedColorToCSSValue } from "@rotext/web-utils";

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
  displayMode: () => RoWidgetDisplayMode;
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
        ? undefined
        : { transform: "rotate(45deg)" }}
      onTouchEnd={props.onTouchEnd}
      onClick={props.onClick}
    />
  );
};
