export {
  BLOCK_EXTENSION_LIST,
  INLINE_EXTENSION_LIST,
  PROSE_CLASS,
  TAG_NAME_MAP,
} from "./consts";
import { TAG_NAME_MAP } from "./consts";

import { registerCustomElement as registerCustomElementForExternalLink } from "./navigation/external-link/mod";
import { registerCustomElementForRefLink } from "./navigation/ref-link/mod";
import { registerCustomElementForWikiLink } from "./navigation/wiki-link/mod";
import { registerCustomElementsForDicexp } from "./dicexp/mod";

import { registerCustomElement as registerCustomElementForBlockCallError } from "./domestic/call-errors/BlockCallError";
import { registerCustomElement as registerCustomElementForInlineCallError } from "./domestic/call-errors/InlineCallError";
import { registerCustomElement as registerCustomElementForScratchOff } from "./domestic/ScratchOff";
import { registerCustomElement as registerCustomElementForCollapse } from "./domestic/Collapse";
import { registerCustomElement as registerCustomElementForCallout } from "./domestic/Callout";
import { registerCustomElement as registerCustomElementForCodeBlock } from "./domestic/CodeBlock";

import { registerCustomElement as registerCustomElementForRotextPreview } from "./domestic-internal/rotext-example/mod";

let hasRegistered = false;

export function registerCustomElementsOnce() {
  if (hasRegistered) return;

  // 内建错误相关。
  registerCustomElementForBlockCallError(TAG_NAME_MAP.block_call_error);
  registerCustomElementForInlineCallError(TAG_NAME_MAP.inline_call_error);

  // 内建行内。
  registerCustomElementForRefLink();
  registerCustomElementForWikiLink();
  registerCustomElementsForDicexp();

  // 块级扩展。
  registerCustomElementForScratchOff(TAG_NAME_MAP.scratch_off);
  registerCustomElementForCollapse(TAG_NAME_MAP.collapse);
  registerCustomElementForCodeBlock(TAG_NAME_MAP.code_block);
  registerCustomElementForCallout(TAG_NAME_MAP.callout);

  // 行内扩展。
  registerCustomElementForExternalLink(TAG_NAME_MAP.external_link);

  // 站点专用的块级扩展。
  registerCustomElementForRotextPreview("x-rotext-example", {
    fixtureTagName: "x-rotext-example-fixture",
  });
}
