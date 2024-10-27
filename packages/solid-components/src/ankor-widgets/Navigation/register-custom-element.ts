import { customElement } from "solid-element";

import {
  createRefLinkComponent,
  CreateRefLinkComponentOptions,
} from "./create-component";

export function registerCustomElement(
  tag: string,
  opts: CreateRefLinkComponentOptions,
) {
  customElement(tag, { address: "" }, createRefLinkComponent(opts));
}
