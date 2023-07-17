import { Component, createEffect, createSignal, on, Setter } from "solid-js";
import { Portal } from "solid-js/web";
import { EditorView } from "codemirror";

import { createCodeMirrorEditor } from "../code-mirror-editor";
import * as storeEditorView from "../../stores/editor-view";

let nextEditorID = 1;

const Editor: Component<
  { text: string; setText: Setter<string>; class?: string }
> = (props) => {
  const editorID = nextEditorID++;

  const [blankHeightAtEnd, setBlankHeightAtEnd] = createSignal(0);

  const { element, view } = createCodeMirrorEditor({
    initialDoc: props.text,
    setDoc: props.setText,
    class: `${props.class} editor-${editorID}`,
    extensions: [EditorView.lineWrapping],
  });

  createEffect(on([storeEditorView.topLine], () => {
    const _view = view();
    const _topLine = clampLine(_view, storeEditorView.topLine());

    const lineBlock = getLineBlock(_view, _topLine);
    const yMargin = -lineBlock.height * (_topLine - (_topLine | 0));
    const scrollEffect = EditorView.scrollIntoView(
      lineBlock.from,
      { y: "start", yMargin },
    );

    _view.dispatch({ effects: [scrollEffect] });
  }));

  createEffect(on([storeEditorView.maxTopLineFromPreview], () => {
    const _maxTopLineFromPreview = storeEditorView.maxTopLineFromPreview();
    if (!_maxTopLineFromPreview) return;

    const _view = view();
    const _maxTopLine = clampLine(
      _view,
      storeEditorView.maxTopLineFromPreview(),
    );

    const scrollEl = findScrollElement(_view.dom);
    if (!scrollEl) return;

    const lineBlock = getLineBlock(_view, _maxTopLine);
    const yMargin = -lineBlock.height * (_maxTopLine - (_maxTopLine | 0));
    const maxOffsetTop = lineBlock.top + yMargin;

    const lastLineBlock = getLineBlock(_view, _view.state.doc.lines);

    const heightUnscrollableFromPreview = Math.max(
      maxOffsetTop + scrollEl.offsetHeight - lastLineBlock.bottom,
      0,
    );
    setBlankHeightAtEnd(heightUnscrollableFromPreview);
  }));

  return (
    <>
      <Portal mount={document.querySelector("head")}>
        <style>
          {blankHeightAtEnd() && `
          .editor-${editorID} .cm-content::after {
            display: block;
            height: ${blankHeightAtEnd()}px; 
            content: "";
          }
        `}
        </style>
      </Portal>
      {element}
    </>
  );
};
export default Editor;

function clampLine(view: EditorView, line: number) {
  return Math.min(view.state.doc.lines, Math.max(line, 1));
}

function getLineBlock(view: EditorView, line: number) {
  const linePos = view.state.doc.line(line | 0).from;
  return view.lineBlockAt(linePos);
}

function findScrollElement(el: HTMLElement) {
  el = el.parentElement;
  while (el) {
    if (el.scrollHeight > el.clientHeight) return el;
    el = el.parentElement;
  }
  return null;
}
