import { Component, createMemo, Index, on, Show } from "solid-js";

import {
  Button,
  DropdownItem,
  Tabs,
  TabWithDropdown,
} from "../../../components/ui/mod";

import { EditorStore } from "../editor-store";
import { textEncoder } from "../../../utils/global";

import examples from "../examples";

import { Solution } from "./EditorWrapper";
import { EditorPartStore } from "./store";

// `Intl.Segmenter` 来自 ES2022，目前（2023/10）Firefox 还不支持
const segmenter: Intl.Segmenter | null = (() => {
  if ("Segmenter" in window.Intl) {
    return new (Intl as any).Segmenter(undefined, { granularity: "grapheme" });
  }
  return null;
})();

const EditorTopBar: Component<{
  store: EditorPartStore;
  editorStore: EditorStore;
}> = (props) => {
  const infoText = createMemo(on([() => props.editorStore.text], ([text]) => {
    const charCount = segmenter ? [...segmenter.segment(text)].length : null;
    const byteCount = textEncoder.encode(text).length;
    const lineCount = text.split("\n").length;

    return [
      ...(charCount !== null ? [`${charCount}字`] : []),
      `${byteCount}字节`,
      `${lineCount}行`,
    ].join(" | ");
  }));

  return (
    <div class="flex h-full justify-between items-center">
      <div>
        <Tabs>
          <TabWithDropdown
            summary={`编辑器（${props.store.solution}）`}
            isActive={true}
          >
            <DropDownItemForSolution
              solutionID="CM6"
              solutionFullName="CodeMirror 6"
              currentSolution={() => props.store.solution}
              setCurrentSolution={(v) => props.store.solution = v}
            />
            <DropDownItemForSolution
              solutionID="ta"
              solutionFullName="textarea"
              currentSolution={() => props.store.solution}
              setCurrentSolution={(v) => props.store.solution = v}
            />
          </TabWithDropdown>
        </Tabs>
      </div>
      <div class="flex items-center gap-1">
        <span class="text-xs text-gray-500">{infoText()}</span>
        <select
          class="w-20 text-xs"
          value="label"
          onChange={(ev) => {
            if (ev.currentTarget.value === "restore") {
              props.editorStore.restoreUserEditedText();
            } else if (ev.currentTarget.value === "clear") {
              props.editorStore.loadText("", { isPreset: true });
            } else {
              props.editorStore.loadText(
                examples.get(ev.currentTarget.value as any),
                { isPreset: true },
              );
            }
            ev.currentTarget.value = "label";
          }}
        >
          <option value="label" disabled selected>
            {props.editorStore.canRestoreUserEditedText()
              ? "恢复/示例/清空"
              : "示例/清空"}
          </option>
          <Show when={props.editorStore.canRestoreUserEditedText()}>
            <option value="restore">
              恢复
            </option>
          </Show>
          <Index each={examples.keys()}>
            {(name) => <option value={name()}>示例：{name()}</option>}
          </Index>
          <option value="clear">
            清空
          </option>
        </select>
      </div>
    </div>
  );
};

export default EditorTopBar;

const DropDownItemForSolution: Component<
  {
    solutionID: Solution;
    solutionFullName: string;
    currentSolution: () => string;
    setCurrentSolution: (v: Solution) => void;
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
