import {
  ComputedColor,
  getComputedColor,
  getComputedCSSValueOfClass,
} from "@rolludejo/web-internal/styling";

let backgroundColorCache: ComputedColor | undefined;
/**
 * XXX: 不能在 ./consts.ts 里以副作用的形式计算颜色并赋值为常量。看起来 “计算所
 * 用样式被插入 HTML” 要晚于 consts.ts 中副作用的执行，因此计算出的颜色会是透明。
 */
export function getBackgroundColor() {
  return backgroundColorCache ??= getComputedColor(
    getComputedCSSValueOfClass("background-color", "tuan-background"),
  )!;
}
