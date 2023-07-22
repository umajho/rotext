import { createSignal } from "solid-js";

/**
 * 用于表示位于编辑器顶部的那一行（浮点数）。
 * 小数点后代表那一行已经经过了多少（`.1` = `10%`）。
 */
export interface TopLine {
  number: number;
  setFrom: "editor" | "preview" | null;
}

export type ActiveLines = [top: number, bottom: number];

export function createEditorStore(initialText: string = "") {
  const [text, setText] = createSignal(initialText);
  const [topLine, setTopLine] = createSignal<TopLine>({
    number: 1,
    setFrom: null,
  });
  // FIXME: 初始值应该在编辑器那边设置
  const [activeLines, setActiveLines] = createSignal<ActiveLines>([1, 1]);

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
    get activeLines(): ActiveLines | null {
      return activeLines();
    },
    set activeLines(v: ActiveLines) {
      setActiveLines(v);
    },
  };
}

export type EditorStore = ReturnType<typeof createEditorStore>;
