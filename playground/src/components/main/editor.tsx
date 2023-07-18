import {
  Component,
  createEffect,
  createMemo,
  createSignal,
  on,
  onMount,
  Setter,
} from "solid-js";
import { Portal } from "solid-js/web";
import { EditorView } from "codemirror";

import { createCodeMirrorEditor } from "../code-mirror-editor";
import * as storeEditorView from "../../stores/editor-view";
import { debounceEventHandler } from "../../utils/mod";

let nextEditorID = 1;

type ScrollHandler = (ev: Event, view: EditorView) => void;

const Editor: Component<
  { text: string; setText: Setter<string>; class?: string }
> = (props) => {
  const editorID = nextEditorID++;

  const [scrollHandler, setScrollHandler] = createSignal<ScrollHandler>();

  const [blankHeightAtEnd, setBlankHeightAtEnd] = createSignal(0);

  // XXX: 由于无法根据滚动事件得知滚动来自用户输入还是由程序控制，
  //      这里创建一个计数器用于记录程序发起的滚动的次数，以此粗略判断事件是否触发自用户输入：
  //      用户滚动时，事件会爆发性地增多，即使处于自动滚动，计数（乐观地）能在短时间内降到 0。
  // XXX: 由于计数器有时不会归零（大概是因为滚动到同一处不会触发事件的缘故），
  //      这里用计时器检查这种情况，在超过时限时手动将其归零。
  const [pendingAutoScrolls, setPendingAutoScrolls] = createAutoResetCounter();

  const scrollHandlerExtension = EditorView.domEventHandlers({
    scroll(event, view) {
      scrollHandler?.()(event, view);
    },
  });

  const { element, view, scrollContainerDOM } = createCodeMirrorEditor({
    initialDoc: props.text,
    setDoc: props.setText,
    class: `${props.class} editor-${editorID}`,
    extensions: [EditorView.lineWrapping, scrollHandlerExtension],
  });

  const contentPadding = createMemo(() => {
    const _view = view();
    if (!_view) return { top: 0, bottom: 0 };
    return getPaddingPixels(_view.contentDOM);
  });

  onMount(() => {
    function handleScroll(
      ev: Event & { target: HTMLElement },
      view: EditorView,
    ) {
      if (ev.target !== scrollContainerDOM()) return;

      if (pendingAutoScrolls() > 0) {
        setPendingAutoScrolls.decrease();
        return;
      }
      setPendingAutoScrolls.reset();

      let scrollTop = Math.max(ev.target.scrollTop - contentPadding()!.top, 0);

      const topLineBlock = view.lineBlockAtHeight(scrollTop);
      const topLineInfo = view.state.doc.lineAt(topLineBlock.from);
      const offsetTop = topLineBlock.top;

      const nextLineInfo = topLineInfo.number + 1 <= view.state.doc.lines
        ? view.state.doc.line(topLineInfo.number + 1)
        : null;
      const nextLineBlock = nextLineInfo && view.lineBlockAt(nextLineInfo.from);
      const nextOffsetTop = nextLineBlock
        ? nextLineBlock.top
        : topLineBlock.bottom;

      const progress = (scrollTop - offsetTop) /
        (nextOffsetTop - offsetTop);
      const line = Math.max(topLineInfo.number + progress, 1);

      storeEditorView.setTopline({ number: line, setFrom: "editor" });
    }

    setScrollHandler(() => debounceEventHandler(handleScroll));
  });

  let lastTopLineFromPreview: number | null = null;
  createEffect(on([storeEditorView.topLine], () => {
    const topLineData = storeEditorView.topLine();
    if (!topLineData.setFrom || topLineData.setFrom === "editor") {
      lastTopLineFromPreview = null;
      return;
    }

    if (lastTopLineFromPreview === topLineData.number) {
      return;
    }
    lastTopLineFromPreview = topLineData.number;

    const _view = view();
    const _topLine = clampLine(_view, topLineData.number);

    const lineBlock = getLineBlock(_view, _topLine);
    const yMargin = -lineBlock.height * (_topLine - (_topLine | 0));
    const scrollEffect = EditorView.scrollIntoView(
      lineBlock.from,
      { y: "start", yMargin },
    );

    setPendingAutoScrolls.increase();
    _view.dispatch({ effects: [scrollEffect] });
  }));

  createEffect(on([storeEditorView.maxTopLineFromPreview], () => {
    const _maxTopLineFromPreview = storeEditorView.maxTopLineFromPreview();
    if (!_maxTopLineFromPreview) return;

    const _view = view();
    const _maxTopLine = clampLine(_view, _maxTopLineFromPreview);

    const scrollEl = scrollContainerDOM();
    if (!scrollEl) return;

    const lineBlock = getLineBlock(_view, _maxTopLine);
    const yMargin = lineBlock.height * (_maxTopLine - (_maxTopLine | 0));
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

function getPaddingPixels(el: HTMLElement) {
  const top = getComputedStyle(el, null).paddingTop;
  const bottom = getComputedStyle(el, null).paddingBottom;

  return { // NOTE: 单位是 px，解析时可以直接忽略
    top: parseFloat(top),
    bottom: parseFloat(bottom),
  };
}

function createAutoResetCounter() {
  const THRESHOLD_MS = 50;

  const [value, _setValue] = createSignal(0);
  let lastChangeTime = 0;
  let checking = false;

  function setValue(v: number) {
    v = Math.max(v, 0);
    _setValue(v);
    lastChangeTime = performance.now();

    if (checking || !v) return;
    checking = true;

    function check() {
      if (value() <= 0) {
        checking = false;
        return;
      }

      if (performance.now() - lastChangeTime >= THRESHOLD_MS) {
        _setValue(0);
        checking = false;
        return;
      }
      setTimeout(check, THRESHOLD_MS);
    }
    setTimeout(check, THRESHOLD_MS);
  }

  function increase() {
    setValue(value() + 1);
  }
  function decrease() {
    setValue(value() - 1);
  }
  function reset() {
    setValue(0);
  }

  return [value, { increase, decrease, reset }] as const;
}
