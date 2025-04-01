import { Component, createMemo, lazy, Show, Suspense } from "solid-js";

import { Loading } from "../../../components/ui/mod";

import { EditorStore } from "../editor-store";

import { EditorPartStore } from "./store";

import TextArea from "./solutions/TextArea";

export type Solution = "CM6" | "ta";

const EditorSolutions = {
  CodeMirror6: lazy(() => import("./solutions/CodeMirror6")),
  TextArea,
};

const EditorWrapper: Component<{
  widthClass: string;
  heightClass: string;
  editorStore: EditorStore;
  store: EditorPartStore;
}> = (props) => {
  const sizeClasses = () => `${props.heightClass} ${props.widthClass}`;

  const currentSolution = createMemo(
    () => {
      switch (props.store.solution) {
        case "CM6":
          return EditorSolutions.CodeMirror6;
        case "ta":
          return EditorSolutions.TextArea;
        default:
          throw new Error("unreachable");
      }
    },
  );

  return (
    <Suspense
      fallback={<LocalLoading sizeClasses={sizeClasses()} />}
    >
      <Show
        when={!props.editorStore.isLoadingText}
        fallback={<LocalLoading sizeClasses={sizeClasses()} />}
      >
        {currentSolution()({
          store: props.editorStore,
          class: `${sizeClasses()} overflow-y-scroll`,
        })}
      </Show>
    </Suspense>
  );
};

const LocalLoading: Component<{ sizeClasses: string }> = (props) => (
  <div class={`flex justify-center items-center ${props.sizeClasses}`}>
    <Loading />
  </div>
);

export default EditorWrapper;
