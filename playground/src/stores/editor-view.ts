import { createSignal } from "solid-js";

/**
 * 位于编辑器最上方的那一行，浮点数，小数点后代表那一行已经经过了多少。
 */
export const [topLine, setTopline] = createSignal(1);

export const [
  maxTopLineFromPreview,
  setMaxTopLineFromPreview,
] = createSignal<number>();
