import { customElement } from "solid-element";

import { createStyleProviderFromCSSText } from "@rolludejo/web-internal";

import {
  createRefLinkComponent,
  CreateRefLinkComponentOptions,
} from "./create-ref-link-component";

import defaultStylesForPrimeContent from "./PrimeContent.default.scss?inline";

const defaultStyleProviderForPrimeContent = //
  createStyleProviderFromCSSText(defaultStylesForPrimeContent);

export function registerCustomElement(
  tag: string,
  opts: CreateRefLinkComponentOptions,
) {
  customElement(tag, { address: "" }, createRefLinkComponent(opts));
}

export function getDefaultStyleProviders(): //
CreateRefLinkComponentOptions["styleProviders"] {
  return {
    forPrimeContent: defaultStyleProviderForPrimeContent,
  };
}
