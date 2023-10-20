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

  return () => {
    const styleEl = document.createElement("style");
    styleEl.appendChild(document.createTextNode(text));
    return styleEl;
  };
}

export function attachStyle(shadowRoot: ShadowRoot, p: StyleProvider) {
  if (typeof p === "function") {
    shadowRoot.appendChild(p());
  } else {
    shadowRoot.adoptedStyleSheets.push(p);
  }
}
