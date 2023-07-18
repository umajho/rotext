import { createSignal } from "solid-js";

interface TopLine {
  number: number;
  setFrom: "editor" | "preview" | null;
}

/**
 * 位于编辑器最上方的那一行，浮点数，小数点后代表那一行已经经过了多少。
 */
export const [topLine, setTopline] = createSignal<TopLine>({
  number: 1,
  setFrom: null,
});

export const [
  maxTopLineFromPreview,
  setMaxTopLineFromPreview,
] = createSignal<number>();
