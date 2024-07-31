import {
  makeOnlyInstance as makeOnlyRotextAdapterInstance,
} from "@rotext/wasm-bindings-adapter";
import initRotextWASM, * as rotextBindings from "rotext_wasm_bindings";

import { LookupListRaw } from "../pages/Playground/preview-parts/Preview/internal-types";

import {
  RotextProcessor,
  RotextProcessorProcessOptions,
  RotextProcessResult,
} from "./mod";

const rotextAdapter = await (async () => {
  await initRotextWASM();
  return makeOnlyRotextAdapterInstance(rotextBindings);
})();

export class RustRotextProcessor implements RotextProcessor {
  process(
    input: string,
    opts: RotextProcessorProcessOptions,
  ): RotextProcessResult {
    try {
      const parsingStart = performance.now();
      console.time("rotext RS (dev)");
      const result = rotextAdapter.parseAndRender(input, {
        tagNameMap: opts.tagNameMap,
      });
      console.timeEnd("rotext RS (dev)");
      const parsingTimeMs = performance.now() - parsingStart;

      if (result[0] === "error") {
        return {
          html: null,
          error: new Error(result[1]),
          parsingTimeMs,
          extraInfos: [],
          lookupListRawCollector: null,
        };
      }

      const output = result[1];

      return {
        html: output.html,
        error: null,
        parsingTimeMs,
        extraInfos: [
          ...(output.devEventsInDebugFormat
            ? [{
              name: "事件",
              content: output.devEventsInDebugFormat,
            }]
            : []),
        ],
        lookupListRawCollector: (targetEl: HTMLElement) => {
          const lookupListRaw: LookupListRaw = [];
          for (const [id, { start, end }] of output.blockIDAndLinesPairs) {
            let element = targetEl.querySelector(
              `[data-block-id="${id}"]`,
            )! as HTMLElement;
            lookupListRaw.push({
              element,
              location: {
                start: { line: start },
                end: { line: end },
              },
            });
          }
          return lookupListRaw;
        },
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
