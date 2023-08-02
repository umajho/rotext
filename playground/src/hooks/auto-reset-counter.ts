import { createSignal } from "solid-js";

export function createAutoResetCounter(thresholdMs = 50) {
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
