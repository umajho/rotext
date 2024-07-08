import { createEffect, createMemo, createSignal, on } from "solid-js";

import { fragment, VNodeChildren } from "snabbdom";

import { toSnabbdomChildren } from "@rotext/to-html";

import { TAG_NAME_MAP } from "../../../utils/custom-elements-registration/mod";
import { formatHTML } from "../../../utils/html-formatting";

export type CurrentOutput =
  | [type: "for-unmodified", html: string]
  | [type: "for-modified", html: string, vNodeChildren: VNodeChildren];

type ExtraPackages = {
  rotextParsing: typeof import("@rotext/parsing");
  pretty: typeof import("pretty");
  snabbdomToHTML: typeof import("snabbdom-to-html");
};

export function createRotextExampleStore(opts: {
  originalInput: string;
  expectedOutputHTMLForOriginalInput: string;
  fixtureNames: string[] | null;
  fixtures: { [fixtureName: string]: string } | null;
}) {
  const [input, setInput] = createSignal(opts.originalInput);
  const [actualOutputForOriginalInput, setActualOutputForOriginalInput] =
    createSignal<string | null>(null);

  function getIsInputUnmodified() {
    return input() === opts.originalInput;
  }

  const [isLoadingExtraPackages, setIsLoadingExtraPackages] = //
    createSignal(false);
  const [isVerifyingOutputOfOriginalInput, setIsVerifyingOutputOfOriginalInput] //
   = createSignal(false);

  const [currentOutput, setCurrentOutput] = createSignal<CurrentOutput>([
    "for-unmodified",
    opts.expectedOutputHTMLForOriginalInput,
  ]);
  createEffect(on([input], async ([input]) => {
    const extraPackages_ = loadExtraPackages();
    let extraPackages: ExtraPackages;
    if (extraPackages_ instanceof Promise) {
      setIsLoadingExtraPackages(true);
      extraPackages = await extraPackages_;
      setIsLoadingExtraPackages(false);
    } else {
      extraPackages = extraPackages_;
    }

    if (getIsInputUnmodified()) {
      parseOriginalInputAndAssign({
        originalInput: input,
        actualOutputForOriginalInput,
        setActualOutputForOriginalInput,
        extraPackages,
      });
      setCurrentOutput([
        "for-unmodified",
        opts.expectedOutputHTMLForOriginalInput,
      ]);
    } else {
      parseOriginalInputAndAssign({
        originalInput: opts.originalInput,
        actualOutputForOriginalInput,
        setActualOutputForOriginalInput,
        extraPackages,
      });
      const [html, vChildren] = parse(input, { extraPackages });
      setCurrentOutput(["for-modified", html, vChildren]);
    }
  }, { defer: true }));

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
      return isLoadingExtraPackages();
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

      (async () => {
        setIsVerifyingOutputOfOriginalInput(true);
        const extraPackages_ = loadExtraPackages();
        let extraPackages: ExtraPackages;
        if (extraPackages_ instanceof Promise) {
          setIsLoadingExtraPackages(true);
          extraPackages = await extraPackages_;
          setIsLoadingExtraPackages(false);
        } else {
          extraPackages = extraPackages_;
        }

        parseOriginalInputAndAssign({
          originalInput: opts.originalInput,
          actualOutputForOriginalInput,
          setActualOutputForOriginalInput,
          extraPackages,
        });
        setIsVerifyingOutputOfOriginalInput(false);
      })();
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

let extraPackages: ExtraPackages | Promise<ExtraPackages> | null = null;

function loadExtraPackages() {
  if (!extraPackages) {
    extraPackages = new Promise(async (resolve) => {
      extraPackages = {
        rotextParsing: await import("@rotext/parsing"),
        pretty: (await import("pretty")).default,
        snabbdomToHTML: (await import("snabbdom-to-html")).default,
      };
      resolve(extraPackages);
    });
  }
  return extraPackages;
}

function parseOriginalInputAndAssign(opts: {
  originalInput: string;
  actualOutputForOriginalInput: () => string | null;
  setActualOutputForOriginalInput: (value: string) => void;
  extraPackages: ExtraPackages;
}) {
  if (opts.actualOutputForOriginalInput() === null) {
    const [html, _] = parse(opts.originalInput, {
      extraPackages: opts.extraPackages,
    });
    opts.setActualOutputForOriginalInput(html);
  }
}

function parse(
  input: string,
  opts: { extraPackages: ExtraPackages },
): [html: string, vNodeChildren: VNodeChildren] {
  const doc = opts.extraPackages.rotextParsing.parse(input);
  const vChildren = toSnabbdomChildren(doc, {
    customElementTagNameMap: TAG_NAME_MAP,
  });
  const html = opts.extraPackages.snabbdomToHTML(fragment(vChildren))
    .slice("<div>".length, -("</div>".length));

  return [
    formatHTML(html, { formatter: opts.extraPackages.pretty }),
    vChildren,
  ];
}

export type RotextExampleStore = ReturnType<typeof createRotextExampleStore>;
