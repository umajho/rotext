import { createStyleProviderFromCSSText } from "@rolludejo/internal-web-shared/shadow-root";

import { mountStyle } from "./utils/mod";

import tuanProse from "./tuan-prose.css?inline";

export const STYLE_TEXT = tuanProse;

export const ID = "tuan-prose";

(() => mountStyle(STYLE_TEXT, { id: ID }))();

export function getStyleElement(): HTMLStyleElement {
  return document.getElementById(ID)! as HTMLStyleElement;
}

export const styleProvider = createStyleProviderFromCSSText(STYLE_TEXT);
