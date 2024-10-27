import {
  registerCustomElementForAnkorWidgetNavigation,
} from "@rotext/solid-components/internal";

import { styleProvider as styleProviderForTuanProse } from "../../../styles/tuan-prose";

import {
  BACKGROUND_COLOR,
  INNER_NO_AUTO_OPEN_CLASS,
  PROSE_CLASS,
  TAG_NAME_MAP,
  WIDGET_OWNER_CLASS,
} from "../consts";

import { createDemoRefContentRenderer } from "./ref-content-demo";

import { styleProvider as styleProviderForPreflight } from "../../../styles/preflight";
import { styleProvider as styleProviderForTailwind } from "../../../styles/tailwind";

export function registerCustomElementForRefLink() {
  registerCustomElementForAnkorWidgetNavigation(TAG_NAME_MAP["ref-link"], {
    baseStyleProviders: [styleProviderForPreflight, styleProviderForTailwind],
    classes: {
      forLabelWrapper: "font-mono underline text-[#789922]", // `#789922` is futaba-green.
    },
    backgroundColor: BACKGROUND_COLOR,
    widgetOwnerClass: WIDGET_OWNER_CLASS,
    innerNoAutoOpenClass: INNER_NO_AUTO_OPEN_CLASS,
    label: ["text", (address) => `>>${address}`],
    innerPreviewRenderer: createDemoRefContentRenderer({
      proseClass: PROSE_CLASS,
      proseStyleProvider: styleProviderForTuanProse,
    }),
  });
}
