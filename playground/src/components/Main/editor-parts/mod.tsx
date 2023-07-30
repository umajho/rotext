import { createMemo, JSX, lazy, Suspense } from "solid-js";

import { Button, Loading } from "../../ui";

import { EditorStore } from "../../../hooks/editor-store";

const Editor = lazy(() => import("./Editor"));

const segmenter: Intl.Segmenter | null = (() => {
  if (window.Intl?.Segmenter) {
    return new Intl.Segmenter(undefined, { granularity: "grapheme" });
  }
  return null;
})();
const textEncoder = new TextEncoder();

export function createEditorParts(
  store: EditorStore,
  opts: { widthClass: string; heightClass: string },
): {
  EditorTopBar: JSX.Element;
  Editor: JSX.Element;
} {
  const editorSizeClass = `${opts.heightClass} ${opts.widthClass}`;

  const charCount = createMemo(() =>
    segmenter ? [...segmenter.segment(store.text)].length : null
  );
  const byteCount = createMemo(() => textEncoder.encode(store.text).length);
  const lineCount = createMemo(() => store.text.split("\n").length);
  const infoText = () =>
    [
      ...(charCount() !== null ? [`${charCount()}字`] : []),
      `${byteCount()}字节`,
      `${lineCount()}行`,
    ].join(" | ");

  return {
    EditorTopBar: (
      <div class="flex h-full justify-end items-center px-4 gap-2">
        <span class="text-xs text-gray-500">{infoText()}</span>
        <Button size="xs" onClick={() => store.text = ""}>清空</Button>
      </div>
    ),
    Editor: (
      <Suspense
        fallback={
          <div class={`flex justify-center items-center ${editorSizeClass}`}>
            <Loading />
          </div>
        }
      >
        <Editor
          store={store}
          class={`${editorSizeClass} overflow-y-scroll`}
        />
      </Suspense>
    ),
  };
}
