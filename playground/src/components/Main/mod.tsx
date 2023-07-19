import { Component, createSignal } from "solid-js";

import EditorCard from "./EditorCard";
import ViewerCard from "./ViewerCard";

import rotextExample from "./example.rotext?raw";

const Main: Component = () => {
  const [text, setText] = createSignal(rotextExample);

  return (
    <main>
      <div
        class={`
        flex justify-center flex-col lg:flex-row
        items-center lg:items-stretch gap-8
      `}
      >
        <EditorCard text={text()} setText={setText} />
        <ViewerCard code={text()} />
      </div>
    </main>
  );
};
export default Main;
