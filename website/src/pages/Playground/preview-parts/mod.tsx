import {
  createEffect,
  createSignal,
  Index,
  JSX,
  lazy,
  Match,
  on,
  Show,
  Switch,
} from "solid-js";

import pretty from "pretty";

import {
  Badge,
  BadgeBar,
  Loading,
  Tab,
  Tabs,
} from "../../../components/ui/mod";

import { EditorStore } from "../../../hooks/editor-store";
import { RotextProcessResult } from "../../../processors/mod";
import rotextProcessors from "../../../global-stores/rotext-processors";

const Preview = lazy(() => import("./Preview/mod"));

type Tab =
  | ["preview"]
  | ["html"]
  | [type: "extra", index: number];

export function createPreviewParts(
  store: EditorStore,
  opts: { widthClass: string; heightClass: string },
): {
  PreviewTopBar: JSX.Element;
  Preview: JSX.Element;
} {
  const [currentTab, setCurrentTab] = createSignal<Tab>(["preview"]);

  const [processResult, setProcessResult] = createSignal<
    RotextProcessResult | null
  >(null);
  createEffect(
    on(
      [() => rotextProcessors.currentProvider, () => store.text],
      ([processorProvider, text]) => {
        if (!processorProvider) {
          setProcessResult(null);
          return;
        }
        const processor = processorProvider();

        setProcessResult(processor.process(text));
      },
    ),
  );

  return {
    PreviewTopBar: (
      <div class="flex h-full justify-between items-center">
        <Tabs>
          <Tab
            isActive={currentTab()[0] === "preview"}
            onClick={() => setCurrentTab(["preview"])}
          >
            预览
          </Tab>
          <Tab
            isActive={currentTab()[0] === "html"}
            onClick={() => setCurrentTab(["html"])}
          >
            HTML
          </Tab>
          <Index each={processResult()?.extraInfos}>
            {(info, i) => (
              <Tab
                isActive={currentTab()[0] === "extra" && currentTab()[1] === i}
                onClick={() => setCurrentTab(["extra", i])}
              >
                {info().name}
              </Tab>
            )}
          </Index>
        </Tabs>
        <BadgeBar>
          <Show when={processResult()?.parsingTimeMs}>
            <Badge>
              解析时间：{`${processResult()!.parsingTimeMs!.toFixed(3)}ms`}
            </Badge>
          </Show>
        </BadgeBar>
      </div>
    ),
    Preview: (
      <div class={`flex flex-col items-center ${opts.heightClass}`}>
        <Show
          when={processResult()}
          fallback={
            <div
              class={`flex justify-center items-center h-full ${opts.widthClass}`}
            >
              <Loading />
            </div>
          }
        >
          {(processResult) => (
            <>
              <Preview
                store={store}
                processResult={processResult()}
                class={`${opts.widthClass} px-4`}
                hidden={currentTab()[0] !== "preview"}
              />
              <Switch>
                <Match
                  when={currentTab()[0] === "html" && processResult().html}
                >
                  {
                    /*
                      由于不明原因，pre 默认的 `whitespace: pre` 会导致其内容与
                      编辑器部分重叠起来，所以手动设为 `whitespace: pre-wrap`。
                    */
                  }
                  <pre class="overflow-scroll whitespace-pre-wrap">
                    <code>
                      {pretty(processResult().html!)}
                    </code>
                  </pre>
                </Match>
                <Match when={currentTab()[0] === "extra"}>
                  <pre class="overflow-scroll whitespace-pre-wrap">
                    {processResult().extraInfos[(currentTab() as Extract<Tab, {0:"extra"}>)[1]]?.content}
                  </pre>
                </Match>
              </Switch>
            </>
          )}
        </Show>
      </div>
    ),
  };
}
