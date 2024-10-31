import { Component } from "solid-js";
import { customElement } from "solid-element";

import { StyleProvider } from "@rolludejo/internal-web-shared/shadow-root";

import { ComputedColor } from "@rolludejo/internal-web-shared/styling";

import { createStepsRepresentationComponent } from "./steps-representation";
import {
  createDicexpComponent,
  type DicexpEvaluatorProvider,
} from "./create-dicexp-component";

import { ErrorAlertComponent } from "./external-components";

export function registerCustomElement(
  tag: string,
  opts: {
    baseStyleProviders: StyleProvider[];
    backgroundColor: ComputedColor;
    evaluatorProvider?: DicexpEvaluatorProvider;
    Loading: Component;
    ErrorAlert: ErrorAlertComponent;
    tagNameForStepsRepresentation: string;
  },
) {
  const DicexpComponent = createDicexpComponent({
    ...opts,
    StepsRepresentation: createStepsRepresentationComponent(
      opts.tagNameForStepsRepresentation,
    ),
  });

  customElement(tag, { code: "", evaluation: null }, DicexpComponent);
}
