import { registerCustomElementForStepsRepresentation } from "@dicexp/solid-components";

import {
  ErrorAlert,
  registerCustomElementForAnkorWidgetDicexp,
} from "@rotext/solid-components/internal";

import { Loading } from "../../components/ui/mod";

import { evaluatorProvider } from "./evaluator-provider";

import { TAG_NAME_MAP } from "../consts";
import { getBackgroundColor } from "../utils";

import { styleProvider as styleProviderForPreflight } from "../../styles/preflight";
import { styleProvider as styleProviderForTailwind } from "../../styles/tailwind";

export function registerCustomElementsForDicexp() {
  registerCustomElementForStepsRepresentation("steps-representation");
  registerCustomElementForAnkorWidgetDicexp(TAG_NAME_MAP["dicexp"], {
    baseStyleProviders: [styleProviderForPreflight, styleProviderForTailwind],
    backgroundColor: getBackgroundColor(),
    evaluatorProvider,
    ErrorAlert,
    Loading,
    tagNameForStepsRepresentation: "steps-representation",
  });
}
