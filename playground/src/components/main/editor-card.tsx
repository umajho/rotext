import { Component, Setter } from "solid-js";

import { Badge, BadgeBar, Card } from "../ui";
import { CodeMirrorEditor } from "../code-mirror-editor";

import { EditorView } from "codemirror";

const segmenter = new Intl.Segmenter(undefined, { granularity: "grapheme" });
const textEncoder = new TextEncoder();

export const EditorCard: Component<
  { text: string; setText: Setter<string> }
> = (props) => {
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
      <CodeMirrorEditor
        doc={props.text}
        setDoc={props.setText}
        class="max-h-[25vh] lg:max-h-none lg:h-full lg:min-h-[20rem] resize-none overflow-y-scroll"
        extensions={[EditorView.lineWrapping]}
      />
    </Card>
  );
};
