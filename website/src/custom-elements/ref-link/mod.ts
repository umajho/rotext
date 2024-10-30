import {
  registerCustomElementForAnkorWidgetNavigation,
} from "@rotext/solid-components/internal";

import { styleProvider as styleProviderForTuanProse } from "../../styles/tuan-prose";

import {
  CLASSES_FOR_NAVIGATION_ACTION,
  PROSE_CLASS,
  TAG_NAME_MAP,
} from "../consts";
import { getBackgroundColor } from "../utils";

import { createDemoRefContentRenderer } from "./ref-content-demo";

import { styleProvider as styleProviderForPreflight } from "../../styles/preflight";
import { styleProvider as styleProviderForTailwind } from "../../styles/tailwind";

export function registerCustomElementForRefLink() {
  registerCustomElementForAnkorWidgetNavigation(TAG_NAME_MAP["ref-link"], {
    baseStyleProviders: [styleProviderForPreflight, styleProviderForTailwind],
    classes: {
      forLabelWrapper: "font-mono underline text-[#789922]", // `#789922` is futaba-green.
      forNavigationAction: CLASSES_FOR_NAVIGATION_ACTION,
    },
    backgroundColor: getBackgroundColor(),
    label: ["text", (address) => `>>${address}`],
    innerPreviewRenderer: createDemoRefContentRenderer({
      proseClass: PROSE_CLASS,
      proseStyleProvider: styleProviderForTuanProse,
    }),
  });
}
