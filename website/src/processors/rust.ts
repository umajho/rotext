// @ts-ignore
import { Idiomorph } from "idiomorph/dist/idiomorph.esm.js";

import {
  makeOnlyInstance as makeOnlyRotextAdapterInstance,
} from "@rotext/wasm-bindings-adapter";
import initRotextWASM, * as rotextBindings from "rotext_wasm_bindings";

import { RotextProcessor, RotextProcessResult } from "./mod";

const rotextAdapter = await (async () => {
  await initRotextWASM();
  return makeOnlyRotextAdapterInstance(rotextBindings);
})();

export class RustRotextProcessor implements RotextProcessor {
  private readonly outputEl: HTMLElement;

  constructor(opts: {
    outputContainerEl: HTMLDivElement;
    contentRootClass: string;
  }) {
    if (opts.outputContainerEl.childNodes.length) {
      throw new Error("output container is not empty!");
    }

    this.outputEl = document.createElement("article");
    this.outputEl.classList.add("relative", opts.contentRootClass);
    opts.outputContainerEl.appendChild(this.outputEl);
  }

  parseAndRender(input: string): RotextProcessResult {
    try {
      console.time("rotext RS (dev)");
      const result = rotextAdapter.parseAndRender(input);
      console.timeEnd("rotext RS (dev)");

      Idiomorph.morph(this.outputEl, result.html, { morphStyle: "innerHTML" });

      return {
        error: null,
        lookupListRaw: [], // TODO!!!
      };
    } catch (e) {
      console.timeEnd("rotext JS");
      if (!(e instanceof Error)) {
        e = new Error(`${e}`);
      }
      return {
        error: e as Error,
        lookupListRaw: [],
      };
    }
  }
}
