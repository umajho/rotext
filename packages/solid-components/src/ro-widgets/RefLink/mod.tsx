import { customElement } from "solid-element";

export type {
  RefAddress,
  RefContentRenderer,
} from "./create-ref-link-component";

import {
  createRefLinkComponent,
  CreateRefLinkComponentOptions,
} from "./create-ref-link-component";

import defaultStyle from "./default.scss?inline";

export function registerCustomElement(
  tag: string,
  opts: CreateRefLinkComponentOptions & {
    withStyle: (tagName: string) => string;
  },
) {
  document.head.appendChild(document.createElement("style"))
    .appendChild(document.createTextNode(opts.withStyle(tag)));

  customElement(tag, { address: "" }, createRefLinkComponent(opts));
}

export function withDefaultStyle(tagName: string) {
  return defaultStyle.replace(/ref-link-tag/g, tagName);
}
