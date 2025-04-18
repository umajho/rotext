import { createEffect, createMemo, createSignal, on } from "solid-js";

import { useRotextProcessorsStore } from "../../../contexts/rotext-processors-store";
import { TAG_NAME_MAP } from "../../mod";
import { formatHTML } from "../../../utils/html-formatting";
import { BLOCK_EXTENSION_LIST, INLINE_EXTENSION_LIST } from "../../consts";
import { RotextProcessorProcessOptions } from "../../../processors/mod";

export type CurrentOutput =
  | [type: "for-unmodified", html: string]
  | [type: "for-modified", html: string];

const formattedExpectedOutputCache: Record<string, string> = {};

const rotextProcessOptions: RotextProcessorProcessOptions = {
  requiresLookupListRaw: false,
  blockExtensionList: BLOCK_EXTENSION_LIST,
  inlineExtensionList: INLINE_EXTENSION_LIST,
  tagNameMap: TAG_NAME_MAP,
};

export function createRotextExampleStore(opts: {
  originalInput: string;
  expectedOutputHTMLForOriginalInput: string;
  fixtureNames: string[] | null;
  fixtures: { [fixtureName: string]: string } | null;
}) {
  const rotextProcessors = useRotextProcessorsStore()!;

  const formattedExpectedOutput =
    (formattedExpectedOutputCache[opts.expectedOutputHTMLForOriginalInput] ??=
      formatHTML(opts.expectedOutputHTMLForOriginalInput));

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

  const [currentOutput, setCurrentOutput] = //
    createSignal<CurrentOutput>(["for-unmodified", formattedExpectedOutput]);

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
            const result = processor.process(
              opts.originalInput,
              rotextProcessOptions,
            );
            if (result.error) {
              throw new Error("TODO!!");
            }
            setActualOutputForOriginalInput(formatHTML(result.html!));
          }
        }

        if (getIsInputUnmodified()) {
          setCurrentOutput(["for-unmodified", formattedExpectedOutput]);
        } else {
          const result = processor.process(input, rotextProcessOptions);
          if (result.error) {
            throw new Error("TODO!!");
          }
          setCurrentOutput(["for-modified", formatHTML(result.html!)]);
        }
      },
      { defer: true },
    ),
  );

  const expectedOutputMatchesActual = createMemo(() => {
    const actual = actualOutputForOriginalInput();
    if (actual === null) return null;
    return actual === formattedExpectedOutput;
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
