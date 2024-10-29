import { registerCustomElementForStepsRepresentation } from "@dicexp/solid-components";

import {
  ErrorAlert,
  getDefaultDicexpStyleProviders,
  registerCustomElementForAnkorWidgetDicexp,
} from "@rotext/solid-components/internal";

import { Loading } from "../../components/ui/mod";

import { evaluatorProvider } from "./evaluator-provider";

import { INNER_NO_AUTO_OPEN_CLASS, TAG_NAME_MAP } from "../consts";
import { getBackgroundColor } from "../utils";

export function registerCustomElementsForDicexp() {
  registerCustomElementForStepsRepresentation("steps-representation");
  registerCustomElementForAnkorWidgetDicexp(TAG_NAME_MAP["dicexp"], {
    styleProviders: getDefaultDicexpStyleProviders(),
    backgroundColor: getBackgroundColor(),
    innerNoAutoOpenClass: INNER_NO_AUTO_OPEN_CLASS,
    evaluatorProvider,
    ErrorAlert,
    Loading,
    tagNameForStepsRepresentation: "steps-representation",
  });
}
