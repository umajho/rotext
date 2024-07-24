import * as snabbdom from "snabbdom";

import { parse } from "@rotext/parsing";
import { toSnabbdomChildren } from "@rotext/to-html";

import { LookupListRaw } from "../pages/Playground/preview-parts/Preview/internal-types";
import { TAG_NAME_MAP } from "../utils/custom-elements-registration/mod";

import { RotextProcessor, RotextProcessResult } from "./mod";

export class OldRotextProcessor implements RotextProcessor {
  private readonly patch: ReturnType<typeof snabbdom.init>;
  private lastNode: HTMLElement | snabbdom.VNode;
  private readonly lookupListRaw: LookupListRaw = [];

  private readonly contentRootClass: string;

  constructor(opts: {
    outputContainerEl: HTMLDivElement;
    contentRootClass: string;
  }) {
    if (opts.outputContainerEl.childNodes.length) {
      throw new Error("output container is not empty!");
    }

    const outputEl = document.createElement("div");
    opts.outputContainerEl.appendChild(outputEl);

    this.patch = snabbdom.init(
      [
        snabbdom.classModule,
        snabbdom.styleModule,
        snabbdom.attributesModule,
        createLocationModule(this.lookupListRaw),
      ],
      undefined,
      { experimental: { fragments: true } },
    );

    this.lastNode = outputEl;

    this.contentRootClass = opts.contentRootClass;
  }

  parseAndRender(input: string): RotextProcessResult {
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

      const classMap: snabbdom.Classes = {
        "relative": true,
        [this.contentRootClass]: true,
      };
      const vNode = snabbdom.h("article", { class: classMap }, vChildren);

      this.patch(this.lastNode, vNode);
      this.lastNode = vNode;

      return {
        error: null,
        lookupListRaw: [...this.lookupListRaw],
        parsingTimeMs,
      };
    } catch (e) {
      console.timeEnd("rotext JS");
      if (!(e instanceof Error)) {
        e = new Error(`${e}`);
      }
      return {
        error: e as Error,
        lookupListRaw: [...this.lookupListRaw],
      };
    }
  }
}

function createLocationModule(lookupListRaw: LookupListRaw) {
  const module = {
    pre: () => {
      lookupListRaw.length = 0;
    },
    create: (_oldVNode: snabbdom.VNode, vnode: snabbdom.VNode) => {
      if (vnode.data?.location) {
        const el = vnode.elm as HTMLElement;
        lookupListRaw.push({
          element: el,
          location: vnode.data.location,
        });
      }
    },
    update: (oldVNode: snabbdom.VNode, vnode: snabbdom.VNode) => {
      module.create(oldVNode, vnode);
    },
  };

  return module;
}
