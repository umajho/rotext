import { JSX, Show } from "solid-js";

import { Button } from "../../../../components/ui/mod";

import { RotextExampleStore } from "../store";

export function createInputParts(
  store: RotextExampleStore,
): { InputTopBar: JSX.Element; InputPane: JSX.Element } {
  return {
    InputTopBar: (
      <div class="flex h-full px-2 items-center">
        <div class="flex-1" />
        <div class="flex items-center">
          <Show when={!store.isInputUnmodified}>
            <div class="flex gap-2 items-center">
              <div class="text-xs text-gray-300 font-bold">
                输入变更
              </div>
              <Button size="xs" onClick={() => store.reset()}>复原</Button>
            </div>
          </Show>
        </div>
      </div>
    ),
    InputPane: (
      <textarea
        class="one-dark one-dark-background w-full h-full px-4 py-2 resize-none"
        // see: https://stackoverflow.com/a/77687417
        // XXX: `field-sizing` 是 2024/03 才有浏览器实现的 CSS 属性，目前
        // （2024/07）浏览器大致支持有六成，Safari 还不支持。考虑到
        // textarea “不会随着内容高度调整自身高度” 也不是不能用，就这样了。
        style="field-sizing: content;"
        value={store.input}
        onInput={(ev) => store.input = ev.currentTarget.value}
      >
      </textarea>
    ),
  };
}
