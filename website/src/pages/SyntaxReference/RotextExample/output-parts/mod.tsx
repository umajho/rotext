import { createSignal, JSX, Match, Switch } from "solid-js";

import { Loading, Tab, Tabs } from "../../../../components/ui/mod";

import { RotextExampleStore } from "../store";

import { Preview, PreviewContent } from "./Preview";

export function createOutputParts(
  store: RotextExampleStore,
): { OutputTopBar: JSX.Element; OutputPane: JSX.Element } {
  const [currentTab, setCurrentTab] = //
    createSignal<"preview" | "source">("preview");

  const previewContentSource = () => store.currentOutput[1];
  const previewContent = (): PreviewContent => {
    switch (store.currentOutput[0]) {
      case "for-unmodified":
        return ["html", previewContentSource()];
      case "for-modified":
        return ["v-node-children", store.currentOutput[2]];
      default:
        throw new Error("unreachable");
    }
  };

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
        <Match when={store.isLoadingForCurrentOutput}>
          <div class="flex w-full h-full justify-center items-center">
            <Loading />
          </div>
        </Match>
        <Match when={currentTab() === "preview"}>
          <Preview content={previewContent} />
        </Match>
        <Match when={currentTab() === "source"}>
          <div class="bg-black w-full h-full overflow-y-scroll">
            <pre class="px-4 py-2">
            <code>{previewContentSource()}</code>
            </pre>
          </div>
        </Match>
      </Switch>
    ),
  };
}
