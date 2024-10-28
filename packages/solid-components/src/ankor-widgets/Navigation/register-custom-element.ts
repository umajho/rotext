import { customElement } from "solid-element";

import {
  createNavigationComponent,
  CreateNavigationComponentOptions,
} from "./create-component";

export function registerCustomElement(
  tag: string,
  opts: CreateNavigationComponentOptions,
) {
  customElement(tag, { address: "" }, createNavigationComponent(opts));
}
