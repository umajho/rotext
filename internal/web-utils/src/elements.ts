export function closestContainer(el: HTMLElement): HTMLElement | null {
  do {
    const display = getComputedStyle(el).display;
    if (["block", "list-item", "table-cell"].indexOf(display) >= 0) return el;
    if (el.parentElement) {
      el = el.parentElement;
    } else {
      const root = el.getRootNode();
      if ("host" in root) {
        el = (root as ShadowRoot).host as HTMLElement;
      } else {
        break;
      }
    }
  } while (true);
  return null;
}
