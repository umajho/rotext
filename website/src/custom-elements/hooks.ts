import { createSignal } from "solid-js";

/**
 * TODO: 去重。（重复声明于 "@rotext/solid-components"。）
 */
interface Watchable<T> {
  currentValue: T;
  onChange: (cb: (value: T) => void) => void;
}

export function createSignalGetterFromWatchable<T>(watchable: Watchable<T>) {
  const [getter, setter] = createSignal(watchable.currentValue);
  watchable.onChange((v) => setter(() => v));

  return getter;
}
