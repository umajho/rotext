import { customElement } from "solid-element";

import { createStyleProviderFromCSSText } from "@rolludejo/web-internal/shadow-root";

import {
  createRefLinkComponent,
  CreateRefLinkComponentOptions,
} from "./create-ref-link-component";

import defaultStylesForLabelContent from "./LabelContent.default.scss?inline";

const defaultStyleProviderForLabelContent = //
  createStyleProviderFromCSSText(defaultStylesForLabelContent);

export function registerCustomElement(
  tag: string,
  opts: CreateRefLinkComponentOptions,
) {
  customElement(tag, { address: "" }, createRefLinkComponent(opts));
}

export function getDefaultStyleProviders(): //
CreateRefLinkComponentOptions["styleProviders"] {
  return {
    forLabelContent: defaultStyleProviderForLabelContent,
  };
}
