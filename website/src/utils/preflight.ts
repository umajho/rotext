import { createStyleProviderFromCSSText } from "@rolludejo/internal-web-shared/shadow-root";

export const styleProviderForPreflight = (() => {
  const preflightEl = document.getElementById("preflight") as HTMLStyleElement;
  return createStyleProviderFromCSSText(preflightEl.innerText);
})();
