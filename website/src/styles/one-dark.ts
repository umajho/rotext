import { createStyleProviderFromCSSText } from "@rolludejo/web-internal";

import { mountStyle } from "./utils/mod";

import oneDark from "./one-dark.scss?inline";

export const STYLE_TEXT = oneDark;

export const ID = "one-dark";

(() => mountStyle(STYLE_TEXT, { id: ID }))();

export function getStyleElement(): HTMLStyleElement {
  return document.getElementById(ID)! as HTMLStyleElement;
}

export const styleProvider = createStyleProviderFromCSSText(STYLE_TEXT);
