import {
  createStyleProviderFromCSSText,
  StyleProvider,
} from "@rolludejo/internal-web-shared/shadow-root";

import styles from "./App.module.css";
import tailwind from "./App.css?inline";

import {
  getComputedColor,
  getComputedCSSValueOfClass,
} from "@rolludejo/internal-web-shared/styling";

export const BACKGROUND_COLOR = getComputedColor(
  getComputedCSSValueOfClass("background-color", styles["App"]!),
)!;

export const baseStyleProviders: StyleProvider[] = [
  // 重复解析两次（一次这里，一次是 `App.tsx` 中的副作用引入）就两次吧，反正只是
  // 个协助开发的小 demo。
  createStyleProviderFromCSSText(tailwind),
];
