import { Component, createSignal, JSX, Match, Show, Switch } from "solid-js";

import { Button, Loading, Tab, Tabs } from "../../../../components/ui/mod";

import { RotextExampleStore } from "../create-store";

import { Preview, PreviewContent } from "./Preview";

export function createOutputParts(
  store: RotextExampleStore,
): { OutputTopBar: JSX.Element; OutputPane: JSX.Element } {
  const [currentTab, setCurrentTab] = //
    createSignal<"preview" | "source" | "actual-preview" | "actual-source">(
      "preview",
    );

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
        <div class="flex-1 flex h-full items-center">
          <Tabs>
            <Tab
              isActive={currentTab() === "preview"}
              onClick={() => setCurrentTab("preview")}
            >
              <Show when={store.expectedOutputMatchesActual === false}>
                <span class="text-blue-500">期待</span>
              </Show>预览
            </Tab>
            <Tab
              isActive={currentTab() === "source"}
              onClick={() => setCurrentTab("source")}
            >
              <Show when={store.expectedOutputMatchesActual === false}>
                <span class="text-blue-500">期待</span>
              </Show>源码
            </Tab>
            <Show when={store.expectedOutputMatchesActual === false}>
              <Tab
                isActive={currentTab() === "actual-preview"}
                onClick={() => setCurrentTab("actual-preview")}
              >
                <span class="text-red-500">实际</span>预览
              </Tab>
              <Tab
                isActive={currentTab() === "actual-source"}
                onClick={() => setCurrentTab("actual-source")}
              >
                <span class="text-red-500">实际</span>源码
              </Tab>
            </Show>
          </Tabs>
        </div>
        <div class="flex gap-2 items-center">
          <Switch>
            <Match when={store.expectedOutputMatchesActual === null}>
              <Button
                size="xs"
                disabled={store.isVerifyingOutputOfOriginalInput()}
                onClick={() => store.verifyOutputOfOriginalInput()}
              >
                验证
              </Button>
            </Match>
            <Match when={store.expectedOutputMatchesActual}>
              <span class="font-bold text-green-500">
                期待输出与实际一致。
              </span>
            </Match>
            <Match when={true}>
              <span class="font-bold text-red-500">
                期待输出与实际不一致！
              </span>
            </Match>
          </Switch>
        </div>
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
          <OutputPaneSourceTab previewContentSource={previewContentSource()} />
        </Match>
        <Match when={currentTab() === "actual-preview"}>
          <Preview
            content={() => ["html", store.actualOutputHTMLForOriginalInput!]}
          />
        </Match>
        <Match when={currentTab() === "actual-source"}>
          <OutputPaneSourceTab
            previewContentSource={store.actualOutputHTMLForOriginalInput!}
          />
        </Match>
      </Switch>
    ),
  };
}

const OutputPaneSourceTab: Component<{
  previewContentSource: string;
}> = (props) => {
  return (
    <div class="bg-black w-full h-full overflow-y-scroll">
      <pre class="px-4 py-2">
  <code>{props.previewContentSource}</code>
      </pre>
    </div>
  );
};
