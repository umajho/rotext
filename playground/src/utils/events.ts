/**
 * see: <https://stackoverflow.com/a/43321596>
 */
export function mouseDownNoDoubleClickToSelect(ev: MouseEvent) {
  if (ev.detail > 1) {
    ev.preventDefault();
  }
}
