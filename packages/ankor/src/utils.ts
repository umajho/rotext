import { findClosestElementEx } from "@rolludejo/internal-web-shared/dom";
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
  return findClosestElementEx(el, (el) => {
    const display = getComputedStyle(el).display;
    return ["block", "list-item", "table-cell"].indexOf(display) >= 0;
  });
}
