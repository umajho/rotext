import { createEffect, createSignal, on } from "solid-js";

import { fragment, VNodeChildren } from "snabbdom";

import { toSnabbdomChildren } from "@rotext/to-html";

import { TAG_NAME_MAP } from "../../../utils/custom-elements-registration/mod";

export type CurrentOutput =
  | [type: "for-unmodified", html: string]
  | [type: "for-modified", html: string, vNodeChildren: VNodeChildren];

export function createRotextExampleStore(opts: {
  originalInput: string;
  originalExpectedOutput: string;
  fixtureNames: string[] | null;
  fixtures: { [fixtureName: string]: string } | null;
}) {
  const [input, setInput] = createSignal(opts.originalInput);

  function getIsInputUnmodified() {
    return input() === opts.originalInput;
  }

  let extraPackages: {
    rotextParsing: typeof import("@rotext/parsing");
    pretty: typeof import("pretty");
    snabbdomToHTML: typeof import("snabbdom-to-html");
  } | null = null;
  const [isLoadingExtraPackages, setIsLoadingExtraPackages] = //
    createSignal(false);

  const [currentOutput, setCurrentOutput] = createSignal<CurrentOutput>([
    "for-unmodified",
    opts.originalExpectedOutput,
  ]);
  createEffect(on([input], async ([input]) => {
    if (getIsInputUnmodified()) {
      setCurrentOutput(["for-unmodified", opts.originalExpectedOutput]);
      return;
    }

    if (!extraPackages) {
      let shouldReturn = false;
      setIsLoadingExtraPackages((prev) => {
        shouldReturn = prev;
        return true;
      });
      if (shouldReturn) return;

      extraPackages = {
        rotextParsing: await import("@rotext/parsing"),
        pretty: (await import("pretty")).default,
        snabbdomToHTML: (await import("snabbdom-to-html")).default,
      };
      setIsLoadingExtraPackages(false);
    }

    if (getIsInputUnmodified()) {
      setCurrentOutput(["for-unmodified", opts.originalExpectedOutput]);
    } else {
      const doc = extraPackages.rotextParsing.parse(input);
      const vChildren = toSnabbdomChildren(doc, {
        customElementTagNameMap: TAG_NAME_MAP,
      });
      const html = extraPackages.snabbdomToHTML(fragment(vChildren))
        .slice("<div>".length, -("</div>".length));
      setCurrentOutput(["for-modified", extraPackages.pretty(html), vChildren]);
    }
  }));

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

    get originalExpectedOutput() {
      return opts.originalExpectedOutput;
    },
    get isLoadingForCurrentOutput() {
      return isLoadingExtraPackages();
    },
    get currentOutput() {
      return currentOutput();
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
