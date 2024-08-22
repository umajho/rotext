import { ComputedColor } from "@rolludejo/web-internal/styling";

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
