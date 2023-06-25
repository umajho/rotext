import { Component, createEffect, onMount, Setter } from "solid-js";

import { basicSetup, EditorView } from "codemirror";
import { Extension } from "@codemirror/state";
import { oneDark } from "@codemirror/theme-one-dark";

export const CodeMirrorEditor: Component<
  {
    class?: string;
    extensions?: Extension[];
    doc: string;
    setDoc: Setter<string>;
  }
> = (
  props,
) => {
  let parentEl: HTMLDivElement;
  let view: EditorView;

  let isEditing = false;
  createEffect(() => {
    const doc = props.doc;
    if (!view) return;
    if (isEditing) {
      isEditing = false;
      return;
    }
    view.dispatch({
      changes: { from: 0, to: view.state.doc.length, insert: doc },
    });
  });

  onMount(() => {
    const extSync = EditorView.updateListener.of((update) => {
      isEditing = true;
      props.setDoc(update.state.doc.toString());
    });

    view = new EditorView({
      doc: props.doc,
      extensions: [basicSetup, oneDark, extSync, ...(props.extensions ?? [])],
      parent: parentEl,
    });
  });

  return <div ref={parentEl} class={`cm-parent ${props.class ?? ""}`} />;
};
