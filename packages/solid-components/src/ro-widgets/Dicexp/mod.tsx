import { Component } from "solid-js";
import { customElement } from "solid-element";

import type { EvaluatingWorkerManager } from "@dicexp/evaluating-worker-manager";

import { ComputedColor } from "@rotext/web-utils";

import { createStepsRepresentationComponent } from "./steps-representation";
import { createDicexpComponent } from "./create-dicexp-component";

import defaultStyle from "./default.scss?inline";

export function registerCustomElement(
  tag: string,
  opts: {
    withStyle: (tagName: string) => string;
    backgroundColor: ComputedColor;
    widgetOwnerClass: string;
    innerNoAutoOpenClass?: string;
    evaluatorProvider: () => Promise<EvaluatingWorkerManager<any>>;
    Loading: Component;
    ErrorAlert: Component<{ error: Error; showsStack: boolean }>;
    tagNameForStepsRepresentation: string;
  },
) {
  const DicexpComponent = createDicexpComponent({
    ...opts,
    StepsRepresentation: createStepsRepresentationComponent(
      opts.tagNameForStepsRepresentation,
    ),
  });

  document.head.appendChild(document.createElement("style"))
    .appendChild(document.createTextNode(opts.withStyle(tag)));

  customElement(tag, { code: "" }, DicexpComponent);
}

export function withDefaultStyle(tagName: string) {
  return defaultStyle.replace(/dicexp-tag/g, tagName);
}
