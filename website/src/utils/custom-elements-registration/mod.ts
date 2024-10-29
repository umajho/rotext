import { registerCustomElementForStepsRepresentation } from "@dicexp/solid-components";

import {
  ErrorAlert,
  getDefaultDicexpStyleProviders,
  registerCustomElementForAnkorWidgetDicexp,
} from "@rotext/solid-components/internal";

import { Loading } from "../../components/ui/mod";
import { registerCustomElement as registerCustomElementForScratchOff } from "../../components/custom-elements/ScratchOff";
import { registerCustomElement as registerCustomElementForCollapse } from "../../components/custom-elements/Collapse";
import { registerCustomElement as registerCustomElementForCodeBlock } from "../../components/custom-elements/CodeBlock";
import { evaluatorProvider } from "./evaluator-provider";

export { PROSE_CLASS, TAG_NAME_MAP } from "./consts";
import { INNER_NO_AUTO_OPEN_CLASS, TAG_NAME_MAP } from "./consts";
import { getBackgroundColor } from "./utils";

import { registerCustomElementForRefLink } from "./ref-link/mod";
import { registerCustomElementForInternalLink } from "./internal-link/mod";

let hasRegistered = false;

export function registerCustomElementsOnce() {
  if (hasRegistered) return;

  registerCustomElementForRefLink();
  registerCustomElementForInternalLink();
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
  registerCustomElementForScratchOff(TAG_NAME_MAP["scratch-off"], {
    innerNoAutoOpenClass: INNER_NO_AUTO_OPEN_CLASS,
  });
  registerCustomElementForCollapse(TAG_NAME_MAP["collapse"]);
  registerCustomElementForCodeBlock(TAG_NAME_MAP["code-block"]);
}
