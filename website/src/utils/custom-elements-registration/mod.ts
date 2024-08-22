import {
  getComputedColor,
  getComputedCSSValueOfClass,
} from "@rolludejo/web-internal/styling";

import { registerCustomElementForStepsRepresentation } from "@dicexp/solid-components";

import {
  ErrorAlert,
  getDefaultDicexpStyleProviders,
  getDefaultRefLinkStyleProviders,
  registerCustomElementForAnkorWidgetDicexp,
  registerCustomElementForAnkorWidgetRefLink,
} from "@rotext/solid-components/internal";

import { styleProvider as styleProviderForTuanProse } from "../../styles/tuan-prose";

import { Loading } from "../../components/ui/mod";
import { registerCustomElement as registerCustomElementForScratchOff } from "../../components/custom-elements/ScratchOff";
import { registerCustomElement as registerCustomElementForCollapse } from "../../components/custom-elements/Collapse";
import { registerCustomElement as registerCustomElementForCodeBlock } from "../../components/custom-elements/CodeBlock";

import { createDemoRefContentRenderer } from "./ref-content-demo";
import { evaluatorProvider } from "./evaluator-provider";

export const TAG_NAME_MAP = {
  "scratch-off": "x-scratch-off",
  "ref-link": "x-ref-link",
  "dicexp": "x-dicexp",
  "collapse": "x-collapse",
  "code-block": "x-code-block",
};

export const WIDGET_OWNER_CLASS = "widget-owner";
export const PROSE_CLASS = "tuan-prose";

const INNER_NO_AUTO_OPEN_CLASS = "inner-no-auto-open";

const BACKGROUND_COLOR = getComputedColor(
  getComputedCSSValueOfClass("background-color", "tuan-background"),
)!;

let hasRegistered = false;

export function registerCustomElementsOnce() {
  if (hasRegistered) return;

  registerCustomElementForAnkorWidgetRefLink(TAG_NAME_MAP["ref-link"], {
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
  registerCustomElementForAnkorWidgetDicexp(TAG_NAME_MAP["dicexp"], {
    styleProviders: getDefaultDicexpStyleProviders(),
    backgroundColor: BACKGROUND_COLOR,
    widgetOwnerClass: WIDGET_OWNER_CLASS,
    innerNoAutoOpenClass: INNER_NO_AUTO_OPEN_CLASS,
    evaluatorProvider,
    ErrorAlert,
    Loading,
    tagNameForStepsRepresentation: "steps-representation",
  });
  registerCustomElementForScratchOff(TAG_NAME_MAP["scratch-off"], {
    innerNoAutoOpenClass: INNER_NO_AUTO_OPEN_CLASS,
  });
  registerCustomElementForCollapse(TAG_NAME_MAP["collapse"]);
  registerCustomElementForCodeBlock(TAG_NAME_MAP["code-block"]);
}
