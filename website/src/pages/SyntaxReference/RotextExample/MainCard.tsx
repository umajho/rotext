import { Component, For, Show } from "solid-js";

import { Card } from "../../../components/ui/mod";

import { RotextExampleStore } from "./create-store";

import { createInputParts } from "./input-parts/mod";
import { createOutputParts } from "./output-parts/mod";

export const MainCard: Component<{
  store: RotextExampleStore;
}> = (props) => {
  const { InputTopBar, InputPane } = createInputParts(props.store);
  const { OutputTopBar, OutputPane } = createOutputParts(props.store);

  return (
    <>
      <div class="relative">
        <div class="absolute z-10 left-4">
          <div class="bg-indigo-800 px-8 py-2 rounded-lg">
            示例
          </div>
        </div>
      </div>
      <div class="px-4 py-6">
        <Card
          class="bg-indigo-800"
          bodyClass="max-sm:px-1 px-4 py-0"
        >
          <div>
            <div class="grid grid-cols-1 xl:grid-cols-2">
              <div class="max-xl:order-1">
                {InputTopBar}
              </div>
              <div class="max-xl:order-3">
                {OutputTopBar}
              </div>
              <div class="max-xl:order-2">
                {InputPane}
              </div>
              <div class="max-xl:order-4">
                {OutputPane}
              </div>
            </div>
            <Show when={props.store.fixtures}>
              {(fixtures) => (
                <details>
                  <summary>示例中用到的其他页面…</summary>
                  <For each={props.store.fixtureNames!}>
                    {(fixtureName) => (
                      <div class="[&:first-child]:mb-2 [&:not(first-child)]:my-2">
                        <div class="flex w-full justify-center">
                          <div class="font-black text-xl">{fixtureName}</div>
                        </div>
                        <textarea
                          class="one-dark one-dark-background w-full px-4 py-2 resize-none"
                          style="field-sizing: content;"
                          disabled
                        >
                          {fixtures()[fixtureName]!}
                        </textarea>
                      </div>
                    )}
                  </For>
                </details>
              )}
            </Show>
          </div>
        </Card>
      </div>
    </>
  );
};
