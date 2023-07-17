import { JSX, onMount, Setter } from "solid-js";

import { basicSetup, EditorView } from "codemirror";
import { Extension } from "@codemirror/state";
import { oneDark } from "@codemirror/theme-one-dark";

export function createCodeMirrorEditor(
  props: {
    class?: string;
    extensions?: Extension[];
    initialDoc: string;
    setDoc: Setter<string>;
  },
): { element: JSX.Element; view: () => EditorView } {
  let parentEl: HTMLDivElement;
  let view: EditorView;

  onMount(() => {
    const extSync = EditorView.updateListener.of((update) => {
      props.setDoc(update.state.doc.toString());
    });

    view = new EditorView({
      doc: props.initialDoc,
      extensions: [basicSetup, oneDark, extSync, ...(props.extensions ?? [])],
      parent: parentEl,
    });
  });

  return {
    element: <div ref={parentEl} class={`cm-parent ${props.class ?? ""}`} />,
    view: () => view,
  };
}
