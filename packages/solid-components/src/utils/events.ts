/**
 * see: <https://stackoverflow.com/a/43321596>
 *
 * FIXME!!: 好像没用？
 */
export function mouseDownNoDoubleClickToSelect(ev: MouseEvent) {
  if (ev.detail > 1) {
    ev.preventDefault();
  }
}
