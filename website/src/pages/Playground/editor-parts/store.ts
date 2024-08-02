import { createSignal } from "solid-js";
import { Solution } from "./EditorWrapper";

export function createEditorPartStore() {
  const [solution, setSolution] = createSignal<Solution>("ce");

  return {
    get solution() {
      return solution();
    },
    set solution(value: Solution) {
      setSolution(value);
    },
  };
}

export type EditorPartStore = ReturnType<typeof createEditorPartStore>;
