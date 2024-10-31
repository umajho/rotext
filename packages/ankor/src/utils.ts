import { ComputedColor } from "@rolludejo/internal-web-shared/styling";

export function mixColor(
  colorA: ComputedColor,
  weightA: number,
  colorB: ComputedColor,
  weightB: number,
) {
  function mixValue(valueA: number, valueB: number): number {
    return (valueA * weightA + valueB * weightB) | 0;
  }
  return new ComputedColor(
    mixValue(colorA.r, colorB.r),
    mixValue(colorA.g, colorB.g),
    mixValue(colorA.b, colorB.b),
    null,
  );
}

export function closestContainer(el: HTMLElement): HTMLElement | null {
  return closest(el, (el) => {
    const display = getComputedStyle(el).display;
    return ["block", "list-item", "table-cell"].indexOf(display) >= 0;
  });
}

export function closest(
  el: HTMLElement,
  predicate: (el: HTMLElement) => boolean,
): HTMLElement | null {
  do {
    if (predicate(el)) return el;
    if (el.parentElement) {
      const slot = el.slot;
      if (slot && "shadowRoot" in el.parentElement) {
        const slotEls = el.parentElement.shadowRoot?.querySelectorAll("slot");
        let hasFound = false;
        for (const slotEl of slotEls ?? []) {
          if (slotEl.name === slot && slotEl.parentElement) {
            hasFound = true;
            el = slotEl.parentElement;
          }
        }
        if (!hasFound) return null;
      } else {
        el = el.parentElement;
      }
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
