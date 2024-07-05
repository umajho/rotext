import "../../../styles/one-dark";

import {
  Component,
  createEffect,
  createSignal,
  JSX,
  on,
  onCleanup,
  onMount,
  Show,
} from "solid-js";

import { ActiveLines, EditorStore, TopLine } from "../../../hooks/editor-store";
import { binarySearch } from "../../../utils/algorithm";
import { debounceEventHandler } from "../../../utils/mod";
import { createAutoResetCounter } from "../../../hooks/auto-reset-counter";

const Editor: Component<{ store: EditorStore; class?: string }> = (props) => {
  let scrollContainerEl!: HTMLDivElement,
    contentContainerEl!: HTMLDivElement;

  let changing = false;

  onMount(() => {
    createEffect(on([() => props.store.text], () => {
      if (changing) {
        changing = false;
        return;
      }
      contentContainerEl.innerHTML = textToInlineHTML(props.store.text);
    }));
  });

  function handleChange(ev: InputEvent) {
    const currentTarget = ev.currentTarget as HTMLDivElement;

    changing = true;

    props.store.text = nodesToText([...currentTarget.childNodes]);
  }

  const {
    beforeInputHandler,
    pasteHandle,
    copyHandler,
    cutHandler,
  } = createBasicEditorFunctionalities(() => contentContainerEl);

  const [highlightElement, setHighlightElement] = createSignal<JSX.Element>();

  const [scrollHandler, setScrollHandler] = createSignal<(ev: Event) => void>();

  const [blankHeightAtEnd, setBlankHeightAtEnd] = createSignal<number>();

  onMount(() => {
    const lookupData = createLookupList({
      textChanged: () => props.store.text,
      scrollContainerSizeChanged: createResizeNotifier(scrollContainerEl),
      contentContainerEl,
    });

    createActiveLinesTracker({
      lookupData,
      contentContainerEl,
      setActiveLines: (v) => props.store.activeLines = v,
    });

    createHighlight({
      activeLines: () => props.store.activeLines,
      lookupData,
      setHighlightElement,
      contentContainerEl,
    });

    const { scrollHandler } = createScrollSyncer({
      topLine: () => props.store.topLine,
      setTopLine: (v) => props.store.topLine = v,
      lookupData,
      scrollContainerEl,
    });
    setScrollHandler(() => debounceEventHandler(scrollHandler));

    createEffect(on([lookupData], () => {
      const lookupData_ = lookupData();
      if (!lookupData_ || !lookupData_.lines.length) return;

      // TODO: 以 “没有折行的单行高度” 作为 “滚动到底部时余留下来的唯一一行的高度”？
      const lastHeight = lookupData_.offsetBottom -
        lookupData_.lines[lookupData_.lines.length - 1]!.offsetTop;
      setBlankHeightAtEnd(
        Math.max(scrollContainerEl.offsetHeight - lastHeight, 0),
      );
    }));
  });

  function handleClickBlankAtEnd() {
    const selection = document.getSelection()!;
    selection.empty();
    const range = new Range();

    const lastChild = contentContainerEl.lastChild;
    if (!lastChild) {
      contentContainerEl.focus();
    } else if (lastChild.nodeType === Node.TEXT_NODE) {
      range.setStart(lastChild, lastChild.nodeValue!.length);
    } else {
      range.setStart(
        contentContainerEl,
        contentContainerEl.childNodes.length - 1,
      );
    }
    selection.addRange(range);
  }

  return (
    <div
      ref={scrollContainerEl}
      class={`one-dark-background ${props.class} overscroll-y-none`}
      onScroll={(ev) => scrollHandler()!(ev)}
    >
      <div class="relative">
        {highlightElement()}
      </div>
      <div
        ref={contentContainerEl}
        class={"editor-ce-content-container" +
          " relative one-dark focus:!outline-none mx-4"}
        contentEditable
        onInput={handleChange}
        onBeforeInput={beforeInputHandler}
        onPaste={pasteHandle}
        onCopy={copyHandler}
        onCut={cutHandler}
      />
      <div
        class="cursor-text"
        style={{ height: `${blankHeightAtEnd()}px` }}
        onClick={handleClickBlankAtEnd}
      />
    </div>
  );
};
export default Editor;

/**
 * @param containerEl 无需追踪，需要是闭包纯粹是因为调用时 ref 尚未被赋值。
 */
function createBasicEditorFunctionalities(containerEl: () => HTMLElement) {
  let draggingFromSelf = false;
  document.addEventListener("dragstart", handleDragStart);
  onCleanup(() => {
    document.removeEventListener("dragstart", handleDragStart);
  });
  function handleDragStart(ev: DragEvent) {
    draggingFromSelf = ev.target === containerEl();
  }

  /**
   * see:
   * - https://stackoverflow.com/a/61237402
   * - https://stackoverflow.com/a/12028136 (剪贴板，没用到)
   */
  function handleBeforeInput(ev: InputEvent) {
    // NOTE: Safari（macOS 16.5）有奇怪的 bug：从 `dataTransfer` `getData`
    //       得到的文本会失去换行。不得不拆分到单独的时间来处理粘贴。
    if (ev.inputType !== "insertText" && ev.dataTransfer) {
      if (ev.inputType !== "insertFromDrop") return;

      // NOTE: 依旧是 Safari（macOS 16.5）的问题：从自己拖入自己时不能用 `insertHTML`
      //       来插入。幸好这时拖动的内容属于自身范围内（文本节点和 <br/>），
      //       因此通过 “遇到这种情况的时候直接结束处理” 来解决。
      if (draggingFromSelf) return;

      ev.preventDefault();

      const text = ev.dataTransfer.getData("text/plain");
      document.execCommand("insertHTML", false, textToInlineHTML(text));
    } else if (ev.inputType === "insertParagraph") {
      ev.preventDefault();
      document.execCommand("insertLineBreak");
    }
  }

  function handlePaste(ev: ClipboardEvent) {
    ev.preventDefault();

    const text = ev.clipboardData!.getData("text/plain");
    document.execCommand("insertHTML", false, textToInlineHTML(text));
  }

  function handleCopy(ev: ClipboardEvent, isCut?: boolean) {
    ev.preventDefault();

    const selection = document.getSelection()!;
    const range = selection.getRangeAt(0);

    const result = rangeToText(range);

    ev.clipboardData!.setData("text/plain", result.text);

    if (isCut) {
      if (result.wholeLine) {
        const range = new Range();
        range.selectNode(result.wholeLine.nextSibling!);
        range.insertNode(result.wholeLine);

        selection.empty();
        selection.addRange(range);
      }
      document.execCommand("delete", false);
    }
  }

  return {
    beforeInputHandler: handleBeforeInput,
    pasteHandle: handlePaste,
    copyHandler: handleCopy,
    cutHandler: (ev: ClipboardEvent) => handleCopy(ev, true),
  };
}

function nodesToText(nodes: Node[]): string {
  return nodes
    .map((node) => {
      if (node.nodeType === Node.TEXT_NODE) return node.nodeValue;
      if (node.nodeType === Node.ELEMENT_NODE) {
        if ((node as Element).tagName === "BR") return "\n";
      }
      return "";
    })
    .join("");
}

function rangeToText(range: Range): { text: string; wholeLine?: Node } {
  if (
    range.startContainer.nodeType === Node.TEXT_NODE &&
    range.startContainer === range.endContainer
  ) {
    const nodeValue = range.startContainer.nodeValue!;
    if (range.startOffset === range.endOffset) {
      // 在某一行没有选择地复制/剪切，代表对象是那一整行
      return { text: nodeValue + "\n", wholeLine: range.startContainer };
    }
    return { text: nodeValue.slice(range.startOffset, range.endOffset) };
  }

  // 从现在起 `range.commonAncestorContainer` 就是 `[contenteditable]` 了
  const container = range.commonAncestorContainer;

  let text = "";
  let startIndex!: number;
  let endIndex!: number;

  if (range.startContainer === container) {
    startIndex = range.startOffset;
  } else {
    const childNodes = container.childNodes;
    for (const [i, node] of childNodes.entries()) {
      if (node === range.startContainer) {
        text = node.nodeValue!.slice(range.startOffset);
        startIndex = i + 1;
        break;
      }
    }
  }

  if (range.endContainer === container) {
    if (!text) { // 始、终都是容器直属的子节点（<br/>），可以直接用 `nodesToText` 处理
      return {
        text: nodesToText(
          [...container.childNodes].slice(startIndex, range.endOffset),
        ),
      };
    }
    endIndex = range.endOffset;
  }

  for (let i = startIndex;; i++) {
    const curNode = container.childNodes[i]!;

    if (
      curNode.nodeType === Node.ELEMENT_NODE &&
      (curNode as Element).tagName === "BR"
    ) {
      text += "\n";
      if (i === endIndex - 1) break;
      continue;
    }

    if (i === endIndex - 1) {
      // NOTE: endIndex 为假时不可能为 0，因为那样肯定已经被先前的同一行时的规则处理掉了
      text += curNode.nodeValue;
      break;
    } else if (curNode === range.endContainer) {
      text += curNode.nodeValue!.slice(0, range.endOffset);
      break;
    }

    text += curNode.nodeValue;
  }

  return { text };
}

function textToInlineHTML(text: string): string {
  const lines = text.split("\n");

  const dummyEl = document.createElement("p");
  for (const [i, line] of lines.entries()) {
    dummyEl.appendChild(document.createTextNode(line));
    if (i < lines.length - 1) {
      dummyEl.appendChild(document.createElement("br"));
    }
  }

  return dummyEl.innerHTML;
}

interface LookupData {
  /** NOTE: 第一行是索引为 0 的元素。 */
  lines: LineData[];
  offsetBottom: number;
}

interface LineData {
  offsetTop: number;
}

function createLookupList(
  opts: {
    textChanged: () => void;
    scrollContainerSizeChanged: () => void;
    contentContainerEl: HTMLElement;
  },
) {
  const [lookupData, setLookupData] = createSignal<LookupData>();

  createEffect(on([opts.textChanged, opts.scrollContainerSizeChanged], () => {
    const lineHeight = getLineHeightPx(opts.contentContainerEl);
    const contentRect = opts.contentContainerEl.getBoundingClientRect();
    const children = opts.contentContainerEl.children;
    const lastNode = opts.contentContainerEl.lastChild;
    const totalLines = children.length +
      (lastNode?.nodeType === Node.TEXT_NODE ? 1 : 0);

    const lines: LineData[] = [{ offsetTop: 0 }];
    for (let i = 0; i < totalLines - 1; i++) {
      const br = children[i]!;
      const brBottom = br.getBoundingClientRect().bottom - contentRect.top;
      const nextLineTop = Math.ceil(brBottom / lineHeight) * lineHeight;
      lines.push({ offsetTop: nextLineTop });
    }

    setLookupData({
      lines,
      offsetBottom: contentRect.bottom - contentRect.top,
    });
  }));

  return lookupData;
}

function getLineHeightPx(el: HTMLElement) {
  return parseFloat(getComputedStyle(el).lineHeight);
}

function getBoundingClientTop(node: Node) {
  const range = new Range();
  range.selectNode(node);
  return range.getBoundingClientRect().top;
}

function createResizeNotifier(el: HTMLElement) {
  const [resized, notifyResize] = createNotifier();
  const observer = new ResizeObserver(notifyResize);
  observer.observe(el);
  onCleanup(() => observer.disconnect());

  return resized;
}

function createNotifier(): [() => void, () => void] {
  const [signal, setSignal] = createSignal<boolean>();

  return [
    () => {
      signal();
      return;
    },
    () => setSignal(!signal()),
  ];
}

function createActiveLinesTracker(
  opts: {
    lookupData: () => LookupData | undefined;
    contentContainerEl: HTMLElement;
    setActiveLines: (v: ActiveLines) => void;
  },
) {
  function handleSelectionChange() {
    if (!opts.contentContainerEl.childNodes.length) return;

    const lookupData_ = opts.lookupData();
    if (!lookupData_) return;

    const selection = document.getSelection();
    if (!selection?.rangeCount) return;
    const range = selection.getRangeAt(0);
    if (
      range.commonAncestorContainer !== opts.contentContainerEl &&
      range.endContainer.parentElement !== opts.contentContainerEl
    ) {
      return;
    }

    const startLineNumber = getLineNumberByY(
      lookupData_.lines,
      getNodeOffestTopInRangeAt(range, "start", opts.contentContainerEl),
    );
    const endLineNumber = getLineNumberByY(
      lookupData_.lines,
      getNodeOffestTopInRangeAt(range, "end", opts.contentContainerEl),
    );

    opts.setActiveLines([startLineNumber, endLineNumber]);
  }

  const debouncedHandler = debounceEventHandler(handleSelectionChange);

  document.addEventListener("selectionchange", debouncedHandler);
  onCleanup(() =>
    document.removeEventListener("selectionchange", debouncedHandler)
  );

  // Safari (macOS, 16.5) 在连续的第二次回车换行起，不会触发 `selectionchange` 事件，
  // 只好 workeraround 一下，每次输入后也掉用一次。
  opts.contentContainerEl.addEventListener("input", debouncedHandler);
  onCleanup(() => {
    opts.contentContainerEl.removeEventListener("input", debouncedHandler);
  });
}

function createHighlight(opts: {
  activeLines: () => ActiveLines | null;
  lookupData: () => LookupData | undefined;
  setHighlightElement: (v: () => JSX.Element) => void;
  contentContainerEl: HTMLElement;
}) {
  const [activeLinesOffsets, setActiveLinesOffsets] = createSignal<
    { topPx: number; bottomPx: number }
  >();

  createEffect(on([opts.lookupData, opts.activeLines], () => {
    const lookupData_ = opts.lookupData();
    if (!lookupData_) return;
    const activeLines_ = opts.activeLines();
    if (!activeLines_) return;

    let [startLine, endLine] = activeLines_;
    startLine = Math.min(startLine, lookupData_.lines.length);
    endLine = Math.min(endLine, lookupData_.lines.length);

    const startLineOffsetTop = lookupData_.lines[startLine - 1]!.offsetTop;
    const endLineOffsetBottom = endLine < lookupData_.lines.length
      ? lookupData_.lines[endLine - 1 + 1]!.offsetTop
      : opts.contentContainerEl.getBoundingClientRect().height;

    setActiveLinesOffsets({
      topPx: startLineOffsetTop,
      bottomPx: endLineOffsetBottom,
    });
  }));

  opts.setHighlightElement(() => (
    <Show when={activeLinesOffsets()}>
      {(offsets) => (
        <div
          class="one-dark-background-active-lines absolute w-full"
          style={{
            top: `${offsets().topPx}px`,
            height: `${offsets().bottomPx - offsets().topPx}px`,
          }}
        />
      )}
    </Show>
  ));
}

function getNodeOffestTopInRangeAt(
  range: Range,
  position: "start" | "end",
  containerEl: HTMLElement,
) {
  const node = range[`${position}Container`] === containerEl
    ? containerEl.childNodes[range[`${position}Offset`]]!
    : range[`${position}Container`]!;

  const containerTop = containerEl.getBoundingClientRect().top;
  const clientTop = getBoundingClientTop(node);
  return clientTop - containerTop;
}

function getLineNumberByY(lines: LineData[], y: number) {
  const index = binarySearch(lines, (item, i) => {
    if (item.offsetTop > y) return "less";

    const nextItem = lines[i + 1];
    if (!nextItem || y < nextItem.offsetTop) return true;
    return "greater";
  }) ?? 0;

  return index + 1;
}

interface ScrollLocal {
  line: number;
  progress: number;
}

function getScrollLocalByY(lookupData: LookupData, y: number): ScrollLocal {
  const line = getLineNumberByY(lookupData.lines, y);
  const lineData = lookupData.lines[line - 1]!;
  const offsetBottom = line < lookupData.lines.length
    ? lookupData.lines[line - 1 + 1]!.offsetTop
    : lookupData.offsetBottom;
  const progress = (y - lineData.offsetTop) /
    (offsetBottom - lineData.offsetTop);

  return { line, progress };
}

function createScrollSyncer(opts: {
  topLine: () => TopLine;
  setTopLine: (v: TopLine) => void;
  lookupData: () => LookupData | undefined;
  scrollContainerEl: HTMLElement;
}) {
  const [pendingAutoScrolls, setPendingAutoScrolls] = createAutoResetCounter();

  function handleScroll() {
    if (pendingAutoScrolls()) {
      setPendingAutoScrolls.decrease();
      return;
    }
    setPendingAutoScrolls.reset();

    // XXX: 没有算入内容顶部与滚动容器顶部之间空隙的高度（目前必定为 0）
    const topY = Math.max(opts.scrollContainerEl.scrollTop, 0);
    const scrollLocal = getScrollLocalByY(opts.lookupData()!, topY);
    const number = scrollLocal.line + scrollLocal.progress;

    if (opts.topLine().number === number) return;
    opts.setTopLine({ number, setFrom: "editor" });
  }

  createEffect(on([opts.topLine], () => {
    const topLine = opts.topLine();
    if (topLine.setFrom === "editor") return;

    const lookupData_ = opts.lookupData();
    if (!lookupData_) return;

    const line = topLine.number;
    const lineInt = line | 0;
    const offsetTop = lookupData_.lines[lineInt - 1]!.offsetTop;
    const offsetBottom = lineInt < lookupData_.lines.length
      ? lookupData_.lines[lineInt - 1 + 1]!.offsetTop
      : lookupData_.offsetBottom;

    // XXX: 没有考虑内容顶部与滚动容器顶部之间有空隙的情况
    const newScrollTop = Math.min(
      offsetTop + (offsetBottom - offsetTop) * (line - lineInt),
      opts.scrollContainerEl.scrollHeight - opts.scrollContainerEl.offsetHeight,
    );

    if (newScrollTop !== opts.scrollContainerEl.scrollTop) {
      setPendingAutoScrolls.increase();
      opts.scrollContainerEl.scrollTo({
        top: newScrollTop,
        behavior: "instant",
      });
    }
  }));

  return {
    scrollHandler: handleScroll,
  };
}
