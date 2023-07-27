import { Accessor, createEffect, createSignal, JSX, onMount } from "solid-js";

import { basicSetup, EditorView } from "codemirror";
import { Extension } from "@codemirror/state";
import { oneDark } from "@codemirror/theme-one-dark";

export function createCodeMirrorEditor(
  props: {
    class?: string;
    extensions?: Extension[];
    doc: () => string;
    setDoc: (doc: string) => void;
  },
): {
  element: JSX.Element;
  view: Accessor<EditorView>;
  scrollContainerDOM: HTMLDivElement;
} {
  let parentEl: HTMLDivElement;
  const [view, setView] = createSignal<EditorView>();

  let dispatchedBySelf = false, changedBySelf = false;
  createEffect(() => {
    const doc = props.doc;

    if (changedBySelf) {
      changedBySelf = false;
      return;
    }

    const view_ = view();
    if (!view_) return;

    dispatchedBySelf = true;
    view_.dispatch({
      changes: { from: 0, to: view_.state.doc.length, insert: doc() },
    });
  });

  onMount(() => {
    const extSync = EditorView.updateListener.of((update) => {
      if (!update.docChanged) return;
      if (dispatchedBySelf) {
        dispatchedBySelf = false;
        return;
      }

      changedBySelf = true;
      props.setDoc(update.state.doc.toString());
    });

    setView(
      new EditorView({
        doc: props.doc(),
        extensions: [basicSetup, oneDark, extSync, ...(props.extensions ?? [])],
        parent: parentEl,
      }),
    );
  });

  return {
    element: (
      <div
        ref={parentEl}
        class={`cm-parent overscroll-none ${props.class ?? ""}`}
      />
    ),
    view,
    scrollContainerDOM: parentEl,
  };
}
