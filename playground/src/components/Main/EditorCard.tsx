import { Component, createMemo, lazy, Show, Suspense } from "solid-js";

import { Card, Loading } from "../ui";
import { EditorStore } from "../../hooks/editor-store";

const Editor = lazy(() => import("./Editor"));

const segmenter: Intl.Segmenter | null = (() => {
  if (window.Intl?.Segmenter) {
    return new Intl.Segmenter(undefined, { granularity: "grapheme" });
  }
  return null;
})();
const textEncoder = new TextEncoder();

const EditorCard: Component<
  { store: EditorStore }
> = (props) => {
  const editorSizeClass =
    "max-h-[25vh] lg:max-h-none lg:h-full lg:min-h-[20rem]";

  const charCount = createMemo(() =>
    segmenter ? [...segmenter.segment(props.store.text)].length : null
  );
  const byteCount = createMemo(() =>
    textEncoder.encode(props.store.text).length
  );
  const lineCount = createMemo(() => props.store.text.split("\n").length);
  const infoText = () =>
    [
      ...(charCount() !== null ? [`${charCount()}字`] : []),
      `${byteCount()}字节`,
      `${lineCount()}行`,
    ].join(" | ");

  return (
    <Card class="w-full max-w-[48rem] lg:w-[36rem] lg:max-h-[80vh]">
      <div class="flex justify-end">
        <span class="text-xs text-gray-500">{infoText()}</span>
      </div>
      <Suspense
        fallback={
          <div class={`flex justify-center items-center ${editorSizeClass}`}>
            <Loading />
          </div>
        }
      >
        <Editor
          store={props.store}
          class={`${editorSizeClass} overflow-y-scroll`}
        />
      </Suspense>
    </Card>
  );
};
export default EditorCard;
