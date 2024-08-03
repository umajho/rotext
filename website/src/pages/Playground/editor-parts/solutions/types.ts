import { Component } from "solid-js";

import { EditorStore } from "../../editor-store";

export type EditorSolution = Component<
  { store: EditorStore; class?: string }
>;
