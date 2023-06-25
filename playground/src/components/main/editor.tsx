import { Component, lazy, Setter } from "solid-js";

import { CodeMirrorEditor } from "../code-mirror-editor";

import { EditorView } from "codemirror";

const Editor: Component<
  { text: string; setText: Setter<string>; class?: string }
> = (props) => {
  return (
    <CodeMirrorEditor
      doc={props.text}
      setDoc={props.setText}
      class={props.class}
      extensions={[EditorView.lineWrapping]}
    />
  );
};
export default Editor;
