import { findClosestElementEx } from "@rolludejo/internal-web-shared/dom";

export function debounceEventHandler<Ev extends Event>(
  handler: (ev: Ev) => void,
) {
  let lastEv: Ev | null = null;
  let handling = false;
  return (ev: Ev) => {
    lastEv = ev;
    if (handling) {
      requestAnimationFrame(() => {
        if (lastEv === ev) {
          handler(ev);
        }
      });
    } else {
      handling = true;
      requestAnimationFrame(() => {
        handler(ev);
        handling = false;
      });
    }
  };
}

export const SUPPORTS_DVH = CSS.supports?.("height", "1dvh");

/**
 * 可能不完全准确，但目前足够了。
 */
export function closestScrollContainer(el: HTMLElement): HTMLElement | null {
  return findClosestElementEx(el, (el) => {
    const overflowY = getComputedStyle(el).overflowY;
    return overflowY === "auto" || overflowY === "scroll";
  });
}
