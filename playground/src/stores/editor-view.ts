import { createSignal } from "solid-js";

interface TopLine {
  number: number;
  setFrom: "editor" | "preview" | null;
}

/**
 * 位于编辑器顶部的那一行（浮点数），小数点后代表那一行已经经过了多少。
 */
export const [topLine, setTopline] = createSignal<TopLine>({
  number: 1,
  setFrom: null,
});

/**
 * 预览内容滚动到底端时，预览顶部对应的那一行（浮点数）。
 *
 * 用于确定编辑器最下方需要多高的空白，才能让预览内容滚动到底端时，其顶端仍能与编辑器保持同步。
 *
 * TODO: 反过来，也有可能是预览内容底端需要额外的空白。
 *       但由于这种情况比较少见（会有吗？），目前先不去实现了。
 */
export const [
  maxTopLineFromPreview,
  setMaxTopLineFromPreview,
] = createSignal<number>();
