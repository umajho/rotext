import { Component } from "solid-js";
import { customElement, noShadowDOM } from "solid-element";

export function registerCustomElement(tag: string) {
  customElement(tag, { input: "", expected: null }, RotextExample);
}

// TODO!!: 改成 widget 那样挂在 widget anchor 上。
export const RotextExample: Component<
  { input: string; expected: string | null }
> = (props) => {
  noShadowDOM();

  return (
    <div class="py-2">
      (TODO!!!)
      <div class="bg-red-500">{props.input}</div>
      <div class="bg-blue-500">{props.expected}</div>
      <div class="bg-green-500">…</div>
    </div>
  );
};
