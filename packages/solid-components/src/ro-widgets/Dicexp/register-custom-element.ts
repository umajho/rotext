import { Component } from "solid-js";
import { customElement } from "solid-element";

import { createStyleProviderFromCSSText } from "@rolludejo/web-internal";

import { ComputedColor } from "@rotext/web-utils";

import { createStepsRepresentationComponent } from "./steps-representation";
import {
  createDicexpComponent,
  CreateDicexpComponentOptions,
  type DicexpEvaluatorProvider,
} from "./create-dicexp-component";

import defaultStylesForPrimeContent from "./PrimeContent.default.scss?inline";
import { ErrorAlertComponent } from "./external-components";

const defaultStyleProviderForPrimeContent = //
  createStyleProviderFromCSSText(defaultStylesForPrimeContent);

export function registerCustomElement(
  tag: string,
  opts: {
    styleProviders: CreateDicexpComponentOptions["styleProviders"];
    backgroundColor: ComputedColor;
    widgetOwnerClass: string;
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

export function getDefaultStyleProviders() {
  return {
    forPrimeContent: defaultStyleProviderForPrimeContent,
  };
}
