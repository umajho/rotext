import { Component, lazy, Match, Suspense, Switch } from "solid-js";

import { Loading } from "../../../components/ui/mod";

import { EditorStore } from "../editor-store";

import ContentEditable from "./EditorByContentEditable";
import TextArea from "./EditorByTextArea";
import { EditorPartStore } from "./store";

export type Solution = "CM6" | "ce" | "ta";

const EditorSolutions = {
  CodeMirror6: lazy(() => import("./EditorByCodeMirror6")),
  ContentEditable,
  TextArea,
};

const EditorWrapper: Component<{
  widthClass: string;
  heightClass: string;
  editorStore: EditorStore;
  store: EditorPartStore;
}> = (props) => {
  const sizeClasses = () => `${props.heightClass} ${props.widthClass}`;

  return (
    <Suspense
      fallback={
        <div class={`flex justify-center items-center ${sizeClasses()}`}>
          <Loading />
        </div>
      }
    >
      <Switch>
        <Match when={props.store.solution === "CM6"}>
          <EditorSolutions.CodeMirror6
            store={props.editorStore}
            class={`${sizeClasses()} overflow-y-scroll`}
          />
        </Match>
        <Match when={props.store.solution === "ce"}>
          <EditorSolutions.ContentEditable
            store={props.editorStore}
            class={`${sizeClasses()} overflow-y-scroll`}
          />
        </Match>
        <Match when={props.store.solution === "ta"}>
          <EditorSolutions.TextArea
            store={props.editorStore}
            class={`${sizeClasses()} overflow-y-scroll`}
          />
        </Match>
      </Switch>
    </Suspense>
  );
};

export default EditorWrapper;
