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

export function createEditorStore(initialText: string | Promise<string>) {
  const [isLoadingText, setIsLoadingText] = createSignal(false);
  const [text, setText] = createSignal("");
  const loadText = (text: string | Promise<string>) => {
    if (isLoadingText()) return;

    if (typeof text === "string") {
      setText(text);
      return;
    }

    setIsLoadingText(true);
    text.then((text) => {
      setText(text);
      setIsLoadingText(false);
    });
  };
  loadText(initialText);

  const [topLine, setTopLine] = createSignal<TopLine>({
    number: 1,
    setFrom: null,
  });
  // FIXME: 初始值应该在编辑器那边设置
  const [activeLines, setActiveLines] = createSignal<ActiveLines | null>(null);

  return {
    get text() {
      return text();
    },
    set text(v: string) {
      setText(v);
    },
    loadText,
    get isLoadingText() {
      return isLoadingText();
    },
    get topLine() {
      return topLine();
    },
    set topLine(v: TopLine) {
      setTopLine(v);
    },
    /**
     * workaround，详见使用了本函数的地方。
     */
    triggerTopLineUpdateForcedly() {
      setTopLine(structuredClone(topLine()));
    },
    get activeLines(): ActiveLines | null {
      return activeLines();
    },
    set activeLines(v: ActiveLines | null) {
      setActiveLines(v);
    },
  };
}

export type EditorStore = ReturnType<typeof createEditorStore>;
