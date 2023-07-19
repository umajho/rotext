import { Component, lazy, Setter, Suspense } from "solid-js";

import { Badge, BadgeBar, Card, Loading } from "../ui";

const Editor = lazy(() => import("./Editor"));

const segmenter = new Intl.Segmenter(undefined, { granularity: "grapheme" });
const textEncoder = new TextEncoder();

const EditorCard: Component<
  { text: string; setText: Setter<string> }
> = (props) => {
  const editorSizeClass =
    "max-h-[25vh] lg:max-h-none lg:h-full lg:min-h-[20rem]";

  const charCount = () => [...segmenter.segment(props.text)].length;
  const byteCount = () => textEncoder.encode(props.text).length;
  const lineCount = () => props.text.split("\n").length;

  return (
    <Card class="w-full max-w-[48rem] lg:w-[36rem] lg:max-h-[80vh]">
      <BadgeBar class="pb-2">
        <Badge>字数：{charCount()}</Badge>
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
