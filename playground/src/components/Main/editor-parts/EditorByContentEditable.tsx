import "./one-dark.scss";

import { Component, createEffect, on, onCleanup, onMount } from "solid-js";

import { EditorStore } from "../../../hooks/editor-store";

const Editor: Component<{ store: EditorStore; class?: string }> = (props) => {
  let el: HTMLDivElement;

  let changing = false;

  onMount(() => {
    createEffect(on([() => props.store.text], () => {
      if (changing) {
        changing = false;
        return;
      }
      el.innerHTML = textToInlineHTML(props.store.text);
    }));
  });

  function handleChange(ev: InputEvent) {
    const currentTarget = ev.currentTarget as HTMLDivElement;

    changing = true;

    props.store.text = nodesToText([...currentTarget.childNodes]);
  }

  let draggingFromSelf = false;
  document.addEventListener("dragstart", handleDragStart);
  onCleanup(() => {
    document.removeEventListener("dragstart", handleDragStart);
  });
  function handleDragStart(ev: DragEvent) {
    draggingFromSelf = ev.target === el;
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

    const text = ev.clipboardData.getData("text/plain");
    document.execCommand("insertHTML", false, textToInlineHTML(text));
  }

  function handleCopy(ev: ClipboardEvent, isCut?: boolean) {
    ev.preventDefault();

    const selection = document.getSelection();
    const range = selection.getRangeAt(0);

    const result = rangeToText(range);

    ev.clipboardData.setData("text/plain", result.text);

    if (isCut) {
      if (result.wholeLine) {
        const range = new Range();
        range.selectNode(result.wholeLine.nextSibling);
        range.insertNode(result.wholeLine);

        selection.empty();
        selection.addRange(range);
      }
      document.execCommand("delete", false);
    }
  }

  return (
    <div
      ref={el}
      class={`one-dark px-4 ${props.class} resize-none focus:!outline-none`}
      contentEditable
      onInput={handleChange}
      onBeforeInput={handleBeforeInput}
      onPaste={handlePaste}
      onCopy={handleCopy}
      onCut={(ev) => handleCopy(ev, true)}
    />
  );
};
export default Editor;

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
    const nodeValue = range.startContainer.nodeValue;
    if (range.startOffset === range.endOffset) {
      // 在某一行没有选择地复制/剪切，代表对象是那一整行
      return { text: nodeValue + "\n", wholeLine: range.startContainer };
    }
    return { text: nodeValue.slice(range.startOffset, range.endOffset) };
  }

  // 从现在起 `range.commonAncestorContainer` 就是 `[contenteditable]` 了
  const container = range.commonAncestorContainer;

  let text = "";
  let startIndex: number;
  let endIndex: number;

  if (range.startContainer === container) {
    startIndex = range.startOffset;
  } else {
    const childNodes = container.childNodes;
    for (const [i, node] of childNodes.entries()) {
      if (node === range.startContainer) {
        text = node.nodeValue.slice(range.startOffset);
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
    const curNode = container.childNodes[i];

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
      text += curNode.nodeValue.slice(0, range.endOffset);
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
