import { createStyleProviderFromCSSText } from "@rotext/web-utils";

export const styleProdiverForPreflight = (() => {
  const preflightEl = document.getElementById("preflight") as HTMLStyleElement;
  return createStyleProviderFromCSSText(preflightEl.innerText);
})();
