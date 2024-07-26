import * as snabbdom from "snabbdom";
import toHTML from "snabbdom-to-html";

import { parse } from "@rotext/parsing";
import { toSnabbdomChildren } from "@rotext/to-html";

import { TAG_NAME_MAP } from "../utils/custom-elements-registration/mod";
import { LookupListRaw } from "../pages/Playground/preview-parts/Preview/internal-types";

import { RotextProcessor, RotextProcessResult } from "./mod";

export class OldRotextProcessor implements RotextProcessor {
  process(input: string): RotextProcessResult {
    try {
      const parsingStart = performance.now();
      console.time("rotext JS");

      const doc = parse(input, {
        softBreakAs: "br",
        recordsLocation: true,
      });

      console.timeLog("rotext JS", "parsed by peggy");

      const vChildren = toSnabbdomChildren(doc, {
        customElementTagNameMap: TAG_NAME_MAP,
      });

      console.timeLog("rotext JS", "transformed to Snabbdom VDOM");
      console.timeEnd("rotext JS");
      const parsingTimeMs = performance.now() - parsingStart;

      return {
        html: toHTML(snabbdom.fragment(vChildren))
          .slice("<div>".length, -("</div>".length)),
        error: null,
        parsingTimeMs,
        extraInfos: [],
        lookupListRawCollector: collectLookupListRaw,
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

function collectLookupListRaw(outputEl: HTMLElement): LookupListRaw {
  const lookupListRaw: LookupListRaw = [];
  for (const el_ of outputEl.querySelectorAll("[data-sourcemap]")) {
    const el = el_ as HTMLElement;
    const [startLnStr, endLnStr] = el.dataset["sourcemap"]!.split("-");
    lookupListRaw.push({
      element: el,
      location: {
        start: { line: Number(startLnStr) },
        end: { line: Number(endLnStr) },
      },
    });
  }
  return lookupListRaw;
}
