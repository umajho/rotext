import { Component, createSignal } from "solid-js";

import EditorCard from "./EditorCard";
import ViewerCard from "./ViewerCard";

import * as examples from "@rotext/example-documentations";
import { createEditorStore } from "../../hooks/editor-store";

const Main: Component = () => {
  const store = createEditorStore(examples.introduction);

  return (
    <main>
      <div
        class={`
        flex justify-center flex-col lg:flex-row
        items-center lg:items-stretch gap-8
      `}
      >
        <EditorCard store={store} />
        <ViewerCard store={store} />
      </div>
    </main>
  );
};
export default Main;
