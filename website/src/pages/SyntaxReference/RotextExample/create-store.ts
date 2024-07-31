import { createEffect, createMemo, createSignal, on } from "solid-js";

import pretty from "pretty";

import { useRotextProcessorsStore } from "../../../contexts/rotext-processors-store";
import { TAG_NAME_MAP } from "../../../utils/custom-elements-registration/mod";

export type CurrentOutput =
  | [type: "for-unmodified", html: string]
  | [type: "for-modified", html: string];

export function createRotextExampleStore(opts: {
  originalInput: string;
  expectedOutputHTMLForOriginalInput: string;
  fixtureNames: string[] | null;
  fixtures: { [fixtureName: string]: string } | null;
}) {
  const rotextProcessors = useRotextProcessorsStore()!;

  const [input, setInput] = createSignal(opts.originalInput);
  const [actualOutputForOriginalInput, setActualOutputForOriginalInput] =
    createSignal<string | null>(null);

  function getIsInputUnmodified() {
    return input() === opts.originalInput;
  }

  const [isLoading, setIsLoading] = createSignal(false);
  const [
    shouldVerifyOutputOfOriginalInput,
    setShouldVerifyOutputOfOriginalInput,
  ] = createSignal(false);
  const isVerifyingOutputOfOriginalInput = () =>
    shouldVerifyOutputOfOriginalInput() && !actualOutputForOriginalInput();

  const [currentOutput, setCurrentOutput] = createSignal<CurrentOutput>([
    "for-unmodified",
    opts.expectedOutputHTMLForOriginalInput,
  ]);

  createEffect(
    on(
      [
        () => rotextProcessors.currentProvider,
        input,
        shouldVerifyOutputOfOriginalInput,
      ],
      ([processorProvider, input, shouldVerify], old) => {
        let shouldUpdateActualOutput = false;
        if (
          shouldVerify && old &&
          old[0] !== processorProvider
        ) {
          shouldUpdateActualOutput = true;
        }

        if (!processorProvider) {
          setIsLoading(true);
          return;
        }
        setIsLoading(false);

        const processor = processorProvider();

        if (
          !getIsInputUnmodified() || shouldVerify ||
          shouldUpdateActualOutput
        ) {
          if (
            shouldUpdateActualOutput || actualOutputForOriginalInput() === null
          ) {
            const result = processor.process(opts.originalInput, {
              requiresLookupListRaw: false,
              tagNameMap: TAG_NAME_MAP,
            });
            if (result.error) {
              throw new Error("TODO!!");
            }
            setActualOutputForOriginalInput(pretty(result.html!));
          }
        }

        if (getIsInputUnmodified()) {
          setCurrentOutput([
            "for-unmodified",
            opts.expectedOutputHTMLForOriginalInput,
          ]);
        } else {
          const result = processor.process(input, {
            requiresLookupListRaw: false,
            tagNameMap: TAG_NAME_MAP,
          });
          if (result.error) {
            throw new Error("TODO!!");
          }
          setCurrentOutput(["for-modified", pretty(result.html!)]);
        }
      },
      { defer: true },
    ),
  );

  const expectedOutputMatchesActual = createMemo(() => {
    const actual = actualOutputForOriginalInput();
    if (actual === null) return null;
    return actual === opts.expectedOutputHTMLForOriginalInput;
  });
  const [
    onOutputOfOriginalInputVerifiedCallback,
    setOnOutputOfOriginalInputVerifiedCallback,
  ] = createSignal<
    | ["unregistered"]
    | ["registered", ((matches: boolean) => void)]
  >(["unregistered"]);
  let hasOutputOfOriginalInputVerifiedCallbackBeenCalled = false;
  createEffect(
    on(
      [expectedOutputMatchesActual, onOutputOfOriginalInputVerifiedCallback],
      ([matches, cb]) => {
        if (matches === null) return;
        if (hasOutputOfOriginalInputVerifiedCallbackBeenCalled) {
          throw new Error("unreachable: callback has already been called");
        }
        switch (cb[0]) {
          case "unregistered":
            return;
          case "registered": {
            cb[1](matches);
            hasOutputOfOriginalInputVerifiedCallbackBeenCalled = true;
            break;
          }
          default:
            throw new Error("unreachable");
        }
      },
    ),
  );

  return {
    get input() {
      return input();
    },
    set input(v: string) {
      setInput(v);
    },
    get isInputUnmodified() {
      return getIsInputUnmodified();
    },

    get expectedOutputHTMLForOriginalInput() {
      return opts.expectedOutputHTMLForOriginalInput;
    },
    get isLoadingForCurrentOutput() {
      return isLoading();
    },
    get currentOutput() {
      return currentOutput();
    },
    get actualOutputHTMLForOriginalInput() {
      return actualOutputForOriginalInput();
    },
    get expectedOutputMatchesActual() {
      return expectedOutputMatchesActual();
    },
    /**
     * XXX: 只要有了实际输出就能验证，因此这个函数做的就是解析出实际输出。
     */
    verifyOutputOfOriginalInput() {
      if (
        actualOutputForOriginalInput() !== null ||
        isVerifyingOutputOfOriginalInput()
      ) return;

      setShouldVerifyOutputOfOriginalInput(true);
    },
    isVerifyingOutputOfOriginalInput() {
      return isVerifyingOutputOfOriginalInput();
    },
    /**
     * 在原始输入的输出被验证后，调用 cb。如果调用本函数时已经完成验证，会直接调
     * 用 cb。cb 只会被调用一次。
     */
    onOutputOfOriginalInputVerified(cb: (matches: boolean) => void) {
      const cbInStore = onOutputOfOriginalInputVerifiedCallback();
      if (cbInStore[0] === "registered") {
        throw new Error(
          "unreachable: cannot set callback: callback has already been registered",
        );
      } else if (hasOutputOfOriginalInputVerifiedCallbackBeenCalled) {
        throw new Error(
          "unreachable: cannot set callback: callback has already been called",
        );
      }
      setOnOutputOfOriginalInputVerifiedCallback(["registered", cb]);
    },

    reset() {
      setInput(opts.originalInput);
    },

    get fixtureNames() {
      return opts.fixtureNames;
    },
    get fixtures() {
      return opts.fixtures;
    },
  };
}

export type RotextExampleStore = ReturnType<typeof createRotextExampleStore>;
