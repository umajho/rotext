import { createEffect, createSignal, JSX, Match, on, Switch } from "solid-js";

import { toSnabbdomChildren } from "@rotext/to-html";

import { TAG_NAME_MAP } from "../../../../utils/custom-elements-registration/mod";

import { Loading, Tab, Tabs } from "../../../../components/ui/mod";

import { RotextExampleStore } from "../store";

import { Preview, PreviewContent } from "./Preview";

export function createOutputParts(
  store: RotextExampleStore,
): { OutputTopBar: JSX.Element; OutputPane: JSX.Element } {
  const [currentTab, setCurrentTab] = //
    createSignal<"preview" | "source">("preview");

  const [previewContent, setPreviewContent] = //
    createSignal<PreviewContent>(["html", store.originalExpected]);

  let parsingPackage: typeof import("@rotext/parsing") | null = null;
  const [isLoadingParsingPackage, setIsLoadingParsingPackage] = //
    createSignal(false);

  createEffect(on([() => store.isInputOriginal], async ([isInputOriginal]) => {
    if (isInputOriginal) {
      setPreviewContent(["html", store.originalExpected]);
      return;
    }
    if (!parsingPackage) {
      if (isLoadingParsingPackage()) return;

      setIsLoadingParsingPackage(true);
      parsingPackage = await import("@rotext/parsing");
      setIsLoadingParsingPackage(false);
    }

    const doc = parsingPackage.parse(store.input);
    const vChildren = toSnabbdomChildren(doc, {
      customElementTagNameMap: TAG_NAME_MAP,
    });
    setPreviewContent(["v-node-children", vChildren]);
    return;
  }));

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
        <Match when={isLoadingParsingPackage()}>
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
            <code>{store.originalExpected}</code>
            </pre>
          </div>
        </Match>
      </Switch>
    ),
  };
}
