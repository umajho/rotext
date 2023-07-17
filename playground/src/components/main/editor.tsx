import { Component, createEffect, lazy, on, Setter } from "solid-js";
import { EditorView } from "codemirror";

import { createCodeMirrorEditor } from "../code-mirror-editor";
import * as storeEditorView from "../../stores/editor-view";

const Editor: Component<
  { text: string; setText: Setter<string>; class?: string }
> = (props) => {
  const { element, view } = createCodeMirrorEditor({
    initialDoc: props.text,
    setDoc: props.setText,
    class: props.class,
    extensions: [EditorView.lineWrapping],
  });

  createEffect(on([storeEditorView.topLine], () => {
    const _view = view();
    const _topLine = Math.min(
      _view.state.doc.lines,
      Math.max(storeEditorView.topLine(), 1),
    );

    const linePos = _view.state.doc.line(_topLine | 0).from;
    const lineBlock = _view.lineBlockAt(linePos);
    const yMargin = -lineBlock.height * (_topLine - (_topLine | 0));
    _view.dispatch({
      effects: [EditorView.scrollIntoView(linePos, { y: "start", yMargin })],
    });
  }));

  return element;
};
export default Editor;
