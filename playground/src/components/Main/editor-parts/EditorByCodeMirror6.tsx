import {
  Component,
  createEffect,
  createMemo,
  createSignal,
  on,
  onMount,
} from "solid-js";
import { Portal } from "solid-js/web";
import { EditorView } from "codemirror";

import { createCodeMirrorEditor } from "../../code-mirror-editor";
import { debounceEventHandler } from "../../../utils/mod";
import { ActiveLines, EditorStore } from "../../../hooks/editor-store";

let nextEditorID = 1;

const Editor: Component<
  { store: EditorStore; class?: string }
> = (props) => {
  const editorID = nextEditorID++;

  const [blankHeightAtEnd, setBlankHeightAtEnd] = createSignal(0);

  // XXX: 由于无法根据滚动事件得知滚动来自用户输入还是由程序控制，
  //      这里创建一个计数器用于记录程序发起的滚动的次数，以此粗略判断事件是否触发自用户输入：
  //      用户滚动时，事件会爆发性地增多，即使处于自动滚动，计数（乐观地）能在短时间内降到 0。
  // XXX: 由于计数器有时不会归零（大概是因为滚动到同一处不会触发事件的缘故），
  //      这里用计时器检查这种情况，在超过时限时手动将其归零。
  const [pendingAutoScrolls, setPendingAutoScrolls] = createAutoResetCounter();

  const activeLinesWatcher = EditorView.updateListener.of((update) => {
    if (!update.selectionSet) return;
    const mainSelection = update.view.state.selection.main;

    const newActiveLines: ActiveLines = [
      update.view.state.doc.lineAt(mainSelection.from).number,
      update.view.state.doc.lineAt(mainSelection.to).number,
    ];
    const oldActiveLines = props.store.activeLines;
    if (
      oldActiveLines && oldActiveLines[0] === newActiveLines[0] &&
      oldActiveLines[1] === newActiveLines[1]
    ) return;

    props.store.activeLines = newActiveLines;
  });

  const { element, view, scrollContainerDOM } = createCodeMirrorEditor({
    doc: () => props.store.text,
    setDoc: (doc: string) => props.store.text = doc,
    class: `${props.class} editor-${editorID}`,
    extensions: [EditorView.lineWrapping, activeLinesWatcher],
  });

  const contentPadding = createMemo(() => {
    const _view = view();
    if (!_view) return { top: 0, bottom: 0 };
    return getPaddingPixels(_view.contentDOM);
  });

  onMount(() => {
    function handleScroll(
      ev: Event & { target: HTMLElement },
    ) {
      const _view = view();
      if (!_view || ev.target !== scrollContainerDOM) return;

      if (pendingAutoScrolls() > 0) {
        setPendingAutoScrolls.decrease();
        return;
      }
      setPendingAutoScrolls.reset();

      let scrollTop = Math.max(ev.target.scrollTop - contentPadding()!.top, 0);

      const topLineBlock = _view.lineBlockAtHeight(scrollTop);
      const topLineInfo = _view.state.doc.lineAt(topLineBlock.from);
      const offsetTop = topLineBlock.top;

      const nextLineInfo = topLineInfo.number + 1 <= _view.state.doc.lines
        ? _view.state.doc.line(topLineInfo.number + 1)
        : null;
      const nextLineBlock = nextLineInfo &&
        _view.lineBlockAt(nextLineInfo.from);
      const nextOffsetTop = nextLineBlock
        ? nextLineBlock.top
        : topLineBlock.bottom;

      const progress = (scrollTop - offsetTop) /
        (nextOffsetTop - offsetTop);
      const line = Math.max(topLineInfo.number + progress, 1);

      props.store.topLine = { number: line, setFrom: "editor" };
    }

    scrollContainerDOM.addEventListener(
      "scroll",
      debounceEventHandler(handleScroll),
    );

    {
      function calculateBlankHeightAtEnd() {
        const _view = view();
        const _maxTopLine = _view.state.doc.lines;

        const scrollEl = scrollContainerDOM;
        if (!scrollEl) return;

        const lineBlock = getLineBlock(_view, _maxTopLine);
        const yMargin = lineBlock.height * (_maxTopLine - (_maxTopLine | 0));
        const maxOffsetTop = lineBlock.top + yMargin;

        const lastLineBlock = getLineBlock(_view, _view.state.doc.lines);

        const heightUnscrollableFromPreview = Math.max(
          maxOffsetTop + scrollEl.offsetHeight - lastLineBlock.bottom -
            contentPadding().bottom,
          0,
        );
        setBlankHeightAtEnd(heightUnscrollableFromPreview);
      }

      calculateBlankHeightAtEnd();
      new ResizeObserver(calculateBlankHeightAtEnd).observe(scrollContainerDOM);
    }

    {
      let justMounted = true;
      let lastTopLineFromPreview: number | null = null;
      createEffect(on([() => props.store.topLine], (_, prev) => {
        const topLineData = props.store.topLine;
        if (justMounted) {
          justMounted = false;
          if (topLineData.number === 1) return;
        } else if (!topLineData.setFrom || topLineData.setFrom === "editor") {
          lastTopLineFromPreview = null;
          return;
        }

        if (lastTopLineFromPreview === topLineData.number) {
          return;
        }
        lastTopLineFromPreview = topLineData.number;

        scrollTopLineTo(view(), topLineData.number, {
          beforeDispatch: () => {
            setPendingAutoScrolls.increase(!prev);
          },
        });
      }));
    }
  });

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

function scrollTopLineTo(
  view: EditorView,
  topLine: number,
  opts?: { beforeDispatch?: () => void },
) {
  topLine = clampLine(view, topLine);

  const lineBlock = getLineBlock(view, topLine);
  const yMargin = -lineBlock.height * (topLine - (topLine | 0));
  const scrollEffect = EditorView.scrollIntoView(
    lineBlock.from,
    { y: "start", yMargin },
  );

  opts?.beforeDispatch?.();
  view.dispatch({ effects: [scrollEffect] });
}

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

  const [value, setValue_] = createSignal(0);
  const [hardValue, setHardValue_] = createSignal(0);

  let lastChangeTime = 0;
  let checking = false;

  function check() {
    if (value() <= 0) {
      checking = false;
      return;
    }

    if (!hardValue() && performance.now() - lastChangeTime >= THRESHOLD_MS) {
      setValue_(0);
      checking = false;
      return;
    }
    setTimeout(check, THRESHOLD_MS);
  }

  function setValue(value: number) {
    value = Math.max(value, 0);
    setValue_(value);
    lastChangeTime = performance.now();

    if (!checking && value) {
      checking = true;
      setTimeout(check, THRESHOLD_MS);
    }
  }

  function setHardValue(hardValue: number) {
    hardValue = Math.max(hardValue, 0);
    setHardValue_(hardValue);
    lastChangeTime = performance.now();

    if (!checking && !hardValue && value()) {
      setTimeout(check, THRESHOLD_MS);
    }
  }

  function increase(hard?: boolean) {
    if (hard) {
      setHardValue(hardValue() + 1);
    } else {
      setValue(value() + 1);
    }
  }
  function decrease() {
    if (hardValue()) {
      setHardValue(hardValue() - 1);
    } else {
      setValue(value() - 1);
    }
  }
  function reset() {
    setValue(0);
    setHardValue(0);
  }

  return [
    () => value() + hardValue(),
    { increase, decrease, reset },
  ] as const;
}
