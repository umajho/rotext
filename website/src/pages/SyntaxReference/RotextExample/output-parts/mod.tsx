import { createSignal, JSX, Match, Switch } from "solid-js";

import { Tab, Tabs } from "../../../../components/ui/mod";

import { RotextExampleStore } from "../store";

import { Preview } from "./Preview";

export function createOutputParts(
  store: RotextExampleStore,
): { OutputTopBar: JSX.Element; OutputPane: JSX.Element } {
  const [currentTab, setCurrentTab] = //
    createSignal<"preview" | "source">("preview");

  return {
    OutputTopBar: (
      <div class="flex h-full px-2 items-center">
        <Tabs>
          <Tab
            isActive={currentTab() === "preview"}
            onClick={() => setCurrentTab("preview")}
          >
            预览
          </Tab>
        </Tabs>
        <Tabs>
          <Tab
            isActive={currentTab() === "source"}
            onClick={() => setCurrentTab("source")}
          >
            源代码
          </Tab>
        </Tabs>
      </div>
    ),
    OutputPane: (
      <Switch>
        <Match when={currentTab() === "preview"}>
          <Preview html={store.originalExpected} />
        </Match>
        <Match when={currentTab() === "source"}>
          <div class="bg-black w-full h-full overflow-y-scroll">
            <pre class="px-4 py-2">
            <code>{store.originalExpected}</code>
            </pre>
          </div>
        </Match>
      </Switch>
    ),
  };
}
