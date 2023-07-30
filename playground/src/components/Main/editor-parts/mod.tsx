import {
  Component,
  createMemo,
  createSignal,
  JSX,
  lazy,
  Match,
  Suspense,
  Switch,
} from "solid-js";

import {
  Button,
  DropdownItem,
  Loading,
  Tab,
  Tabs,
  TabWithDropdown,
} from "../../ui";

import { EditorStore } from "../../../hooks/editor-store";

const EditorSolutions = {
  CodeMirror6: lazy(() => import("./EditorByCodeMirror6")),
  TextArea: lazy(() => import("./EditorByTextArea")),
  ContentEditable: lazy(() => import("./EditorByContentEditable")),
};

const segmenter: Intl.Segmenter | null = (() => {
  if (window.Intl?.Segmenter) {
    return new Intl.Segmenter(undefined, { granularity: "grapheme" });
  }
  return null;
})();
const textEncoder = new TextEncoder();

export function createEditorParts(
  store: EditorStore,
  opts: { widthClass: string; heightClass: string },
): {
  EditorTopBar: JSX.Element;
  Editor: JSX.Element;
} {
  const editorSizeClass = `${opts.heightClass} ${opts.widthClass}`;

  const charCount = createMemo(() =>
    segmenter ? [...segmenter.segment(store.text)].length : null
  );
  const byteCount = createMemo(() => textEncoder.encode(store.text).length);
  const lineCount = createMemo(() => store.text.split("\n").length);
  const infoText = () =>
    [
      ...(charCount() !== null ? [`${charCount()}字`] : []),
      `${byteCount()}字节`,
      `${lineCount()}行`,
    ].join(" | ");

  const [solution, setSolution] = createSignal<"CM6" | "ce" | "ta">("CM6");

  return {
    EditorTopBar: (
      <div class="flex h-full justify-between items-center">
        <div>
          <Tabs>
            <TabWithDropdown
              summary={`编辑器（${solution()}）`}
              isActive={true}
            >
              <DropDownItemForSolution
                solutionID="CM6"
                solutionFullName="CodeMirror 6"
                currentSolution={solution}
                setCurrentSolution={setSolution}
              />
              <DropDownItemForSolution
                solutionID="ce"
                solutionFullName="contenteditable"
                currentSolution={solution}
                setCurrentSolution={setSolution}
              />
              <DropDownItemForSolution
                solutionID="ta"
                solutionFullName="textarea"
                currentSolution={solution}
                setCurrentSolution={setSolution}
              />
            </TabWithDropdown>
          </Tabs>
        </div>
        <div class="flex items-center">
          <span class="text-xs text-gray-500">{infoText()}</span>
          <Button size="xs" onClick={() => store.text = ""}>清空</Button>
        </div>
      </div>
    ),
    Editor: (
      <Suspense
        fallback={
          <div class={`flex justify-center items-center ${editorSizeClass}`}>
            <Loading />
          </div>
        }
      >
        <Switch>
          <Match when={solution() === "CM6"}>
            <EditorSolutions.CodeMirror6
              store={store}
              class={`${editorSizeClass} overflow-y-scroll`}
            />
          </Match>
          <Match when={solution() === "ce"}>
            <EditorSolutions.ContentEditable
              store={store}
              class={`${editorSizeClass} overflow-y-scroll`}
            />
          </Match>
          <Match when={solution() === "ta"}>
            <EditorSolutions.TextArea
              store={store}
              class={`${editorSizeClass} overflow-y-scroll`}
            />
          </Match>
        </Switch>
      </Suspense>
    ),
  };
}

export const DropDownItemForSolution: Component<
  {
    solutionID: string;
    solutionFullName: string;
    currentSolution: () => string;
    setCurrentSolution: (v: string) => void;
  }
> = (props) => {
  return (
    <DropdownItem>
      <Button
        type="ghost"
        size="sm"
        class="normal-case"
        active={props.currentSolution() === props.solutionID}
        onClick={() => props.setCurrentSolution(props.solutionID)}
      >
        {props.solutionFullName} ({props.solutionID})
      </Button>
    </DropdownItem>
  );
};
