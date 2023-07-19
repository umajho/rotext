import { Component, createMemo, lazy, Setter, Show, Suspense } from "solid-js";

import { Card, Loading } from "../ui";

const Editor = lazy(() => import("./Editor"));

const segmenter: Intl.Segmenter | null = (() => {
  if (window.Intl?.Segmenter) {
    return new Intl.Segmenter(undefined, { granularity: "grapheme" });
  }
  return null;
})();
const textEncoder = new TextEncoder();

const EditorCard: Component<
  { text: string; setText: Setter<string> }
> = (props) => {
  const editorSizeClass =
    "max-h-[25vh] lg:max-h-none lg:h-full lg:min-h-[20rem]";

  const charCount = createMemo(() =>
    segmenter ? [...segmenter.segment(props.text)].length : null
  );
  const byteCount = createMemo(() => textEncoder.encode(props.text).length);
  const lineCount = createMemo(() => props.text.split("\n").length);
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
          text={props.text}
          setText={props.setText}
          class={`${editorSizeClass} overflow-y-scroll`}
        />
      </Suspense>
    </Card>
  );
};
export default EditorCard;
