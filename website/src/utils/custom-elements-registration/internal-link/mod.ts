import {
  registerCustomElementForAnkorWidgetNavigation,
} from "@rotext/solid-components/internal";

import { styleProvider as styleProviderForTuanProse } from "../../../styles/tuan-prose";

import {
  INNER_NO_AUTO_OPEN_CLASS,
  PROSE_CLASS,
  TAG_NAME_MAP,
  WIDGET_OWNER_CLASS,
} from "../consts";
import { getBackgroundColor } from "../utils";

import { createDemoPreviewRenderer } from "./preview-demo";

import { styleProvider as styleProviderForPreflight } from "../../../styles/preflight";
import { styleProvider as styleProviderForTailwind } from "../../../styles/tailwind";

export function registerCustomElementForInternalLink() {
  registerCustomElementForAnkorWidgetNavigation(TAG_NAME_MAP["internal-link"], {
    baseStyleProviders: [styleProviderForPreflight, styleProviderForTailwind],
    classes: {
      forLabelWrapper: "underline text-blue-600",
    },
    backgroundColor: getBackgroundColor(),
    widgetOwnerClass: WIDGET_OWNER_CLASS,
    innerNoAutoOpenClass: INNER_NO_AUTO_OPEN_CLASS,
    label: ["slot"],
    innerPreviewRenderer: createDemoPreviewRenderer({
      proseClass: PROSE_CLASS,
      proseStyleProvider: styleProviderForTuanProse,
    }),
  });
}
