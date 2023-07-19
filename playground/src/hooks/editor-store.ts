import { createSignal } from "solid-js";

/**
 * 用于表示位于编辑器顶部的那一行（浮点数）。
 * 小数点后代表那一行已经经过了多少（`.1` = `10%`）。
 */
interface TopLine {
  number: number;
  setFrom: "editor" | "preview" | null;
}

export function createEditorStore(initialText: string = "") {
  const [text, setText] = createSignal(initialText);
  const [topLine, setTopLine] = createSignal<TopLine>({
    number: 1,
    setFrom: null,
  });

  return {
    get text() {
      return text();
    },
    set text(v: string) {
      setText(v);
    },
    get topLine() {
      return topLine();
    },
    set topLine(v: TopLine) {
      setTopLine(v);
    },
  };
}

export type EditorStore = ReturnType<typeof createEditorStore>;
