export type StyleProvider =
  | (() => HTMLLinkElement | HTMLStyleElement)
  | CSSStyleSheet;

export function createStyleProviderFromCSSText(text: string): StyleProvider {
  try {
    const sheet = new CSSStyleSheet();
    sheet.replace(text);
    return sheet;
  } catch {}

  try {
    const blob = new Blob([text], { type: "text/css" });
    // 由于样式会一直存在，就不考虑 revoke 了
    const objectURL = URL.createObjectURL(blob);
    return () => {
      const linkEl = document.createElement("link");
      linkEl.rel = "stylesheet";
      linkEl.href = objectURL!;
      return linkEl;
    };
  } catch {}

  return () => createStyleElementFromCSSText(text);
}

function createStyleElementFromCSSText(text: string): HTMLStyleElement {
  const styleEl = document.createElement("style");
  styleEl.appendChild(document.createTextNode(text));
  return styleEl;
}

export function adoptStyle(shadowRoot: ShadowRoot, p: StyleProvider) {
  if (typeof p === "function") {
    shadowRoot.appendChild(p());
  } else {
    if ("adoptedStyleSheets" in shadowRoot) {
      shadowRoot.adoptedStyleSheets.push(p);
    } else {
      const cssText = [...p.cssRules].map((r) => r.cssText).join("\n");
      const styleEl = createStyleElementFromCSSText(cssText);
      (shadowRoot as ShadowRoot).appendChild(styleEl);
    }
  }
}
