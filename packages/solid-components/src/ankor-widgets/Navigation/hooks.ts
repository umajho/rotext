import { createEffect, on } from "solid-js";
import { untrack } from "solid-js/web";

export interface Watchable<T> {
  currentValue: T;
  onChange: (cb: (value: T) => void) => void;
}

export function createWatchableFromSignalGetter<T>(
  getter: () => T,
): Watchable<T> {
  const cbs: ((value: T) => void)[] = [];
  const w: Watchable<T> = {
    currentValue: untrack(getter),
    onChange: (cb) => cbs.push(cb),
  };
  createEffect(on([getter], ([v]) => {
    w.currentValue = v;
    cbs.forEach((cb) => cb(v));
  }));

  return w;
}
