import "./one-dark.scss";

import { Component } from "solid-js";

import { EditorStore } from "../../../hooks/editor-store";

const Editor: Component<{ store: EditorStore; class?: string }> = (props) => {
  function handleChange(ev: InputEvent) {
    props.store.text = (ev.currentTarget as HTMLTextAreaElement).value;
  }

  return (
    <textarea
      class={`one-dark px-4 ${props.class} resize-none focus:!outline-none`}
      value={props.store.text}
      onInput={handleChange}
    />
  );
};
export default Editor;
