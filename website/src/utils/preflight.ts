import { createStyleProviderFromCSSText } from "@rolludejo/web-internal/shadow-root";

export const styleProviderForPreflight = (() => {
  const preflightEl = document.getElementById("preflight") as HTMLStyleElement;
  return createStyleProviderFromCSSText(preflightEl.innerText);
})();
