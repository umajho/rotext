import { createEffect, createSignal, on } from "solid-js";

import { RotextProcessor } from "../processors/mod";

export type RotextProcessorName = "old" | "rust";

export type RotextProcessorProvider = () => RotextProcessor;

const CURRENT_ROTEXT_PROCESSOR_NAME_LOCAL_KEY = "currentRotextProcessorName";

const cache: {
  [name in RotextProcessorName]?: RotextProcessorProvider;
} = {};

function createRotextProcessorsStore(
  currentProcessorName: RotextProcessorName,
) {
  const [processorName, setProcessorName] = //
    createSignal<RotextProcessorName>(currentProcessorName);
  const [isBusy, setIsBusy] = createSignal(false);
  const [currentProvider, setCurrentProvider] = createSignal<
    RotextProcessorProvider | null
  >(null);

  createEffect(on([processorName], ([processorName]) => {
    if (isBusy()) {
      throw new Error("busy!");
    }

    if (cache[processorName]) {
      const provider = cache[processorName];
      setCurrentProvider(() => provider);
      return;
    }

    setIsBusy(true);
    setCurrentProvider(null);

    (async () => {
      switch (processorName) {
        case "old":
          {
            const module = await import("../processors/old");
            cache[processorName] = () => new module.OldRotextProcessor();
          }
          break;
        case "rust":
          {
            const module = await import("../processors/rust");
            cache[processorName] = () => new module.RustRotextProcessor();
          }
          break;
        default:
          throw new Error("unreachable");
      }

      setCurrentProvider(() => cache[processorName]!);
      setIsBusy(false);
    })();
  }));

  return {
    switchProcessor(name: RotextProcessorName) {
      localStorage.setItem(CURRENT_ROTEXT_PROCESSOR_NAME_LOCAL_KEY, name);
      setProcessorName(name);
    },

    get currentProviderName(): RotextProcessorName {
      return processorName();
    },

    get currentProvider(): RotextProcessorProvider | null {
      return currentProvider();
    },

    get isBusy(): boolean {
      return isBusy();
    },
  };
}

function getCurrentRotextProcessorNameInLocalStorage(): RotextProcessorName {
  const item = localStorage.getItem(
    CURRENT_ROTEXT_PROCESSOR_NAME_LOCAL_KEY,
  );
  if (item === "old") return item;
  if (item === "rust") return item;
  return "old";
}

export default createRotextProcessorsStore(
  getCurrentRotextProcessorNameInLocalStorage(),
);
