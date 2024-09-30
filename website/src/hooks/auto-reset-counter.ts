import { createSignal } from "solid-js";

/**
 * XXX: 在我也不确定原因的特定情况下，Safari 从调用 `scrollTo` 到报告滚动事件的
 * 间隔会持续高达 50 毫秒，因此 `thresholdMs` 设置成 `50` 会导致该情况下对应事件
 * 监听器获取的计数总是为 0。因此，这里将 `thresholdMs` 的默认值设置为 `100`。
 */
export function createAutoResetCounter(thresholdMs = 100) {
  const [value, setValue_] = createSignal(0);
  const [hardValue, setHardValue_] = createSignal(0);

  let lastChangeTime = 0;
  let checking = false;

  function check() {
    if (value() <= 0) {
      checking = false;
      return;
    }

    if (!hardValue() && performance.now() - lastChangeTime >= thresholdMs) {
      setValue_(0);
      checking = false;
      return;
    }
    setTimeout(check, thresholdMs);
  }

  function setValue(value: number) {
    value = Math.max(value, 0);
    setValue_(value);
    lastChangeTime = performance.now();

    if (!checking && value) {
      checking = true;
      setTimeout(check, thresholdMs);
    }
  }

  function setHardValue(hardValue: number) {
    hardValue = Math.max(hardValue, 0);
    setHardValue_(hardValue);
    lastChangeTime = performance.now();

    if (!checking && !hardValue && value()) {
      setTimeout(check, thresholdMs);
    }
  }

  function increase(hard?: boolean) {
    if (hard) {
      setHardValue(hardValue() + 1);
    } else {
      setValue(value() + 1);
    }
  }
  function decrease() {
    if (hardValue()) {
      setHardValue(hardValue() - 1);
    } else {
      setValue(value() - 1);
    }
  }
  function reset() {
    setValue(0);
    setHardValue(0);
  }

  return [
    () => value() + hardValue(),
    { increase, decrease, reset },
  ] as const;
}
