export type StyleProvider =
  | (() => HTMLLinkElement | HTMLStyleElement)
  | CSSStyleSheet;

export function createStyleProviderFromCSSText(text: string): StyleProvider {
  try {
    const sheet = new CSSStyleSheet();
    sheet.replace(text);
    return sheet;
  } catch {
    return () => {
      const styleEl = document.createElement("style");
      styleEl.appendChild(document.createTextNode(text));
      return styleEl;
    };
  }
}

export function attachStyle(shadowRoot: ShadowRoot, p: StyleProvider) {
  if (typeof p === "function") {
    shadowRoot.appendChild(p());
  } else {
    shadowRoot.adoptedStyleSheets.push(p);
  }
}
