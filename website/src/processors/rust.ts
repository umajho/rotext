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
      const parsingStart = performance.now();
      console.time("rotext RS (dev)");

      const result = rotextAdapter.parseAndRender(input);

      console.timeEnd("rotext RS (dev)");
      const parsingTimeMs = performance.now() - parsingStart;

      return {
        html: result.html,
        error: null,
        parsingTimeMs,
        extraInfos: [
          ...(result.devEventsInDebugFormat
            ? [{
              name: "事件",
              content: result.devEventsInDebugFormat,
            }]
            : []),
        ],
        lookupListRawCollector: () => [],
      };
    } catch (e) {
      console.timeEnd("rotext JS");
      if (!(e instanceof Error)) {
        e = new Error(`${e}`);
      }
      return {
        html: null,
        error: e as Error,
        parsingTimeMs: null,
        extraInfos: [],
        lookupListRawCollector: null,
      };
    }
  }
}
