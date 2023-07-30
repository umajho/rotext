import {
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

  const [solution, setSolution] = createSignal<"CM6" | "ta">("CM6");

  return {
    EditorTopBar: (
      <div class="flex h-full justify-between items-center">
        <div>
          <Tabs>
            <TabWithDropdown
              summary={`编辑器（${solution()}）`}
              isActive={true}
            >
              <DropdownItem>
                <Button
                  type="ghost"
                  size="sm"
                  class="normal-case"
                  active={solution() === "CM6"}
                  onClick={() => setSolution("CM6")}
                >
                  CodeMirror 6 (CM6)
                </Button>
              </DropdownItem>
              <DropdownItem>
                <Button
                  type="ghost"
                  size="sm"
                  class="normal-case"
                  active={solution() === "ta"}
                  onClick={() => setSolution("ta")}
                >
                  textarea (ta)
                </Button>
              </DropdownItem>
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
