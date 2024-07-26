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
  process(input: string): RotextProcessResult {
    try {
      console.time("rotext RS (dev)");
      const result = rotextAdapter.parseAndRender(input);
      console.timeEnd("rotext RS (dev)");

      return {
        html: result.html,
        error: null,
      };
    } catch (e) {
      console.timeEnd("rotext JS");
      if (!(e instanceof Error)) {
        e = new Error(`${e}`);
      }
      return {
        html: null,
        error: e as Error,
      };
    }
  }
}
