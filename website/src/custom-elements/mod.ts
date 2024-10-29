export { PROSE_CLASS, TAG_NAME_MAP } from "./consts";
import { INNER_NO_AUTO_OPEN_CLASS, TAG_NAME_MAP } from "./consts";

import { registerCustomElementForRefLink } from "./ref-link/mod";
import { registerCustomElementForInternalLink } from "./internal-link/mod";
import { registerCustomElementsForDicexp } from "./dicexp/mod";

import { registerCustomElement as registerCustomElementForScratchOff } from "./domestic/ScratchOff";
import { registerCustomElement as registerCustomElementForCollapse } from "./domestic/Collapse";
import { registerCustomElement as registerCustomElementForCodeBlock } from "./domestic/CodeBlock";

let hasRegistered = false;

export function registerCustomElementsOnce() {
  if (hasRegistered) return;

  registerCustomElementForRefLink();
  registerCustomElementForInternalLink();
  registerCustomElementsForDicexp();

  registerCustomElementForScratchOff(TAG_NAME_MAP["scratch-off"], {
    innerNoAutoOpenClass: INNER_NO_AUTO_OPEN_CLASS,
  });
  registerCustomElementForCollapse(TAG_NAME_MAP["collapse"]);
  registerCustomElementForCodeBlock(TAG_NAME_MAP["code-block"]);
}
