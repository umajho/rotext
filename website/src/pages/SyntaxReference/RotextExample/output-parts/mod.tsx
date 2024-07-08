import { createEffect, createSignal, JSX, Match, on, Switch } from "solid-js";

import { fragment } from "snabbdom";

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
    createSignal<PreviewContent>(["html", store.originalExpectedOutput]);
  const [previewContentSource, setPreviewContentSource] = //
    createSignal<string>(store.originalExpectedOutput);

  let extraPackages: {
    rotextParsing: typeof import("@rotext/parsing");
    pretty: typeof import("pretty");
    snabbdomToHTML: typeof import("snabbdom-to-html");
  } | null = null;
  const [isLoadingExtraPackages, setIsLoadingExtraPackages] = //
    createSignal(false);

  createEffect(on([() => store.isInputOriginal], async ([isInputOriginal]) => {
    if (isInputOriginal) {
      setPreviewContent(["html", store.originalExpectedOutput]);
      setPreviewContentSource(store.originalExpectedOutput);
      return;
    }
    if (!extraPackages) {
      if (isLoadingExtraPackages()) return;

      setIsLoadingExtraPackages(true);
      extraPackages = {
        rotextParsing: await import("@rotext/parsing"),
        pretty: (await import("pretty")).default,
        snabbdomToHTML: (await import("snabbdom-to-html")).default,
      };
      setIsLoadingExtraPackages(false);
    }

    const doc = extraPackages.rotextParsing.parse(store.input);
    const vChildren = toSnabbdomChildren(doc, {
      customElementTagNameMap: TAG_NAME_MAP,
    });
    const html = extraPackages.snabbdomToHTML(fragment(vChildren))
      .slice("<div>".length, -("</div>".length));
    setPreviewContentSource(extraPackages.pretty(html));
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
        <Match when={isLoadingExtraPackages()}>
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
