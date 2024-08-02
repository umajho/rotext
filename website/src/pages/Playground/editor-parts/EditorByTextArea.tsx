import "../../../styles/one-dark";

import { Component, createEffect, on } from "solid-js";

import { EditorStore } from "../editor-store";

const Editor: Component<{ store: EditorStore; class?: string }> = (props) => {
  props.store.activeLines = null;

  function handleChange(ev: InputEvent) {
    props.store.text = (ev.currentTarget as HTMLTextAreaElement).value;
  }

  createEffect(on(() => props.store.text, (cur, prev) => {
    if (!prev || cur.length <= prev.length || !cur.startsWith(prev)) return;

    const newLines = cur.slice(prev.length).split("\n");
    if (newLines.length > 2) return;
    if (newLines.length === 2 && !cur.endsWith("\n")) return;

    // 文本的改变是：在最后添加了文本，还可能在然后进行了一次换行。
    props.store.topLine = { number: Infinity, setFrom: "editor" };
  }));

  return (
    <textarea
      class={`one-dark one-dark-background px-4 ${props.class} resize-none focus:!outline-none`}
      value={props.store.text}
      onInput={handleChange}
    />
  );
};
export default Editor;
