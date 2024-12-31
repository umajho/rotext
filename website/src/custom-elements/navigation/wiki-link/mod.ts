import {
  registerCustomElementForAnkorWidgetNavigation,
} from "@rotext/solid-components/internal";

import { styleProvider as styleProviderForTuanProse } from "../../../styles/tuan-prose";

import {
  CLASSES_FOR_NAVIGATION_ACTION,
  PROSE_CLASS,
  TAG_NAME_MAP,
} from "../../consts";
import { getBackgroundColor } from "../../utils";

import { createDemoPreviewRenderer } from "./preview";

import { styleProvider as styleProviderForPreflight } from "../../../styles/preflight";
import { styleProvider as styleProviderForTailwind } from "../../../styles/tailwind";

export function registerCustomElementForWikiLink() {
  registerCustomElementForAnkorWidgetNavigation(TAG_NAME_MAP["wiki-link"], {
    baseStyleProviders: [styleProviderForPreflight, styleProviderForTailwind],
    classes: {
      forLabelWrapper: "underline text-blue-600",
      forNavigationAction: CLASSES_FOR_NAVIGATION_ACTION,
    },
    backgroundColor: getBackgroundColor(),
    label: ["slot"],
    innerPreviewRenderer: createDemoPreviewRenderer({
      proseClass: PROSE_CLASS,
      proseStyleProvider: styleProviderForTuanProse,
    }),
  });
}
