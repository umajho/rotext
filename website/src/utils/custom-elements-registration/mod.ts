import { registerCustomElementForStepsRepresentation } from "@dicexp/solid-components";

import {
  getComputedColor,
  getComputedCSSValueOfClass,
} from "@rotext/web-utils";
import {
  ErrorAlert,
  getDefaultDicexpStyleProviders,
  getDefaultRefLinkStyleProviders,
  registerCustomElementForRoWidgetDicexp,
  registerCustomElementForRoWidgetRefLink,
} from "@rotext/solid-components/internal";

import { styleProvider as styleProviderForTuanProse } from "../../styles/tuan-prose";

import { Loading } from "../../components/ui/mod";
import { registerCustomElement as registerCustomElementForScratchOff } from "../../components/custom-elements/ScratchOff";

import { createDemoRefContentRenderer } from "./ref-content-demo";
import { evaluatorProvider } from "./evaluator-provider";

export const WIDGET_OWNER_CLASS = "widget-owner";
export const PROSE_CLASS = "tuan-prose";

const INNER_NO_AUTO_OPEN_CLASS = "inner-no-auto-open";

const BACKGROUND_COLOR = getComputedColor(
  getComputedCSSValueOfClass("background-color", "tuan-background"),
)!;

let hasRegistered = false;

export function registerCustomElementsOnce() {
  if (hasRegistered) return;

  registerCustomElementForRoWidgetRefLink("x-ref-link", {
    styleProviders: getDefaultRefLinkStyleProviders(),
    backgroundColor: BACKGROUND_COLOR,
    widgetOwnerClass: WIDGET_OWNER_CLASS,
    innerNoAutoOpenClass: INNER_NO_AUTO_OPEN_CLASS,
    refContentRenderer: createDemoRefContentRenderer({
      proseClass: PROSE_CLASS,
      proseStyleProvider: styleProviderForTuanProse,
    }),
  });
  registerCustomElementForStepsRepresentation("steps-representation");
  registerCustomElementForRoWidgetDicexp("x-dicexp-preview", {
    styleProviders: getDefaultDicexpStyleProviders(),
    backgroundColor: BACKGROUND_COLOR,
    widgetOwnerClass: WIDGET_OWNER_CLASS,
    innerNoAutoOpenClass: INNER_NO_AUTO_OPEN_CLASS,
    evaluatorProvider,
    ErrorAlert,
    Loading,
    tagNameForStepsRepresentation: "steps-representation",
  });
  registerCustomElementForScratchOff("x-scratch-off", {
    innerNoAutoOpenClass: INNER_NO_AUTO_OPEN_CLASS,
  });
}
