import { Component } from "solid-js";

import { EditorStore } from "../../../hooks/editor-store";

const Editor: Component<{ store: EditorStore; class?: string }> = (props) => {
  function handleChange(ev: InputEvent) {
    props.store.text = (ev.currentTarget as HTMLTextAreaElement).value;
  }

  return (
    <textarea
      class={`${props.class} resize-none`}
      value={props.store.text}
      onInput={handleChange}
    />
  );
};
export default Editor;
