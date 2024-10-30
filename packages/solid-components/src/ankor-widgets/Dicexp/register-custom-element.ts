import { Component } from "solid-js";
import { customElement } from "solid-element";

import { StyleProvider } from "@rolludejo/web-internal/shadow-root";

import { ComputedColor } from "@rolludejo/web-internal/styling";

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
    innerNoAutoOpenClass?: string;
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
