import { Component, lazy, Setter, Show, Suspense } from "solid-js";

import { Badge, BadgeBar, Card, Loading } from "../ui";

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

  const charCount = () =>
    segmenter ? [...segmenter.segment(props.text)].length : null;
  const byteCount = () => textEncoder.encode(props.text).length;
  const lineCount = () => props.text.split("\n").length;

  return (
    <Card class="w-full max-w-[48rem] lg:w-[36rem] lg:max-h-[80vh]">
      <BadgeBar class="pb-2">
        <Show when={segmenter}>
          <Badge>字数：{charCount()}</Badge>
        </Show>
        <Badge>字节数：{byteCount()}</Badge>
        <Badge>行数：{lineCount()}</Badge>
      </BadgeBar>
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
