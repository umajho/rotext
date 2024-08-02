import { createSignal } from "solid-js";
import { RotextProcessResult } from "../../../processors/mod";

export type Tab =
  | ["preview"]
  | ["html"]
  | [type: "extra", index: number];

export function createPreviewPartStore() {
  const [currentTab, setCurrentTab] = createSignal<Tab>(["preview"]);

  const [processResult, setProcessResult] = createSignal<
    RotextProcessResult | null
  >(null);

  return {
    get currentTab() {
      return currentTab();
    },
    set currentTab(value: Tab) {
      setCurrentTab(value);
    },
    get processResult(): RotextProcessResult | null {
      return processResult();
    },
    set processResult(value: RotextProcessResult) {
      setProcessResult(value);
    },
  };
}

export type PreviewPartStore = ReturnType<typeof createPreviewPartStore>;
