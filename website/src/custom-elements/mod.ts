export { PROSE_CLASS, TAG_NAME_MAP } from "./consts";
import { TAG_NAME_MAP } from "./consts";

import { registerCustomElementForRefLink } from "./navigation/ref-link/mod";
import { registerCustomElementForWikiLink } from "./navigation/wiki-link/mod";
import { registerCustomElementsForDicexp } from "./dicexp/mod";

import { registerCustomElement as registerCustomElementForBlockCallError } from "./domestic/BlockCallError";
import { registerCustomElement as registerCustomElementForScratchOff } from "./domestic/ScratchOff";
import { registerCustomElement as registerCustomElementForCollapse } from "./domestic/Collapse";
import { registerCustomElement as registerCustomElementForCodeBlock } from "./domestic/CodeBlock";

import { registerCustomElement as registerCustomElementForRotextPreview } from "./domestic-internal/rotext-example/mod";

let hasRegistered = false;

export function registerCustomElementsOnce() {
  if (hasRegistered) return;

  registerCustomElementForBlockCallError(TAG_NAME_MAP["block-call-error"]);

  registerCustomElementForRefLink();
  registerCustomElementForWikiLink();
  registerCustomElementsForDicexp();

  registerCustomElementForScratchOff(TAG_NAME_MAP["scratch-off"]);
  registerCustomElementForCollapse(TAG_NAME_MAP["collapse"]);
  registerCustomElementForCodeBlock(TAG_NAME_MAP["code-block"]);

  registerCustomElementForRotextPreview("x-rotext-example");
}
