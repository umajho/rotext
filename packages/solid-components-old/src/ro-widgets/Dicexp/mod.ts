import { Component } from "solid-js";
import { customElement } from "solid-element";

import { createStepsRepresentationComponent } from "../internal/steps-representation";
import { createDicexpComponent } from "./create-dicexp-component";

export function registerCustomElement(
  tag: string,
  opts: {
    dicexpImporter: () => Promise<typeof import("dicexp")>;
    EvaluatingWorker: new () => Worker;
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

  customElement(tag, { code: "" }, DicexpComponent);
}
