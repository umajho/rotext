import { createEffect, createSignal, on } from "solid-js";

import { RotextProcessor } from "../processors/mod";

export type RotextProcessorName = "old" | "rust";

export type RotextProcessorProvider = () => RotextProcessor;

const cache: {
  [name in RotextProcessorName]?: RotextProcessorProvider;
} = {};

export function createRotextProcessorsStore(
  opts: {
    initialProcessorName: RotextProcessorName;
    onCurrentProcessorNameChange: (newName: RotextProcessorName) => void;
  },
) {
  const [processorName, setProcessorName] = //
    createSignal<RotextProcessorName>(opts.initialProcessorName);
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
      setProcessorName(name);
      opts.onCurrentProcessorNameChange(name);
    },

    get currentName(): RotextProcessorName {
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

export type RotextProcessorsStore = ReturnType<
  typeof createRotextProcessorsStore
>;
