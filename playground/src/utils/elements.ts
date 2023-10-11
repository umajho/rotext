export function closestContainer(el: HTMLElement): HTMLElement | null {
  let el_: HTMLElement | null = el;
  do {
    const display = getComputedStyle(el_).display;
    if (["block", "list-item", "table-cell"].indexOf(display) >= 0) return el_;
    el_ = el_.parentElement;
  } while (el_);
  return null;
}
