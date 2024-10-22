export function mountStyle(style: string, opts: { id: string }) {
  if (!document.getElementById(opts.id)) {
    const styleEl = document.createElement("style");
    styleEl.id = opts.id;
    document.head.append(styleEl);
  }

  const styleEl = document.getElementById(opts.id)!;
  if (styleEl.innerText !== style) {
    styleEl.innerText = style;
  }
}

export function mustGetStyleProviderFormDocument(id: string): CSSStyleSheet {
  const sheet = [...document.styleSheets]
    .find((sheet) => {
      if (!sheet.ownerNode) return false;
      return (sheet.ownerNode as HTMLElement).id === id;
    });
  if (!sheet) throw new Error(`Could not find style sheet with id \`${id}\``);
  return sheet;
}
