import {
  getComputedColor,
  getComputedCSSValueOfClass,
} from "@rolludejo/web-internal/styling";

export const TAG_NAME_MAP = {
  "scratch-off": "x-scratch-off",
  "ref-link": "x-ref-link",
  "dicexp": "x-dicexp",
  "collapse": "x-collapse",
  "code-block": "x-code-block",
  "internal-link": "x-internal-link",
};

export const WIDGET_OWNER_CLASS = "widget-owner";
export const PROSE_CLASS = "tuan-prose";

export const INNER_NO_AUTO_OPEN_CLASS = "inner-no-auto-open";

export const BACKGROUND_COLOR = getComputedColor(
  getComputedCSSValueOfClass("background-color", "tuan-background"),
)!;
