import {
  createEffect,
  createMemo,
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
import { useRotextProcessorsStore } from "../../../contexts/rotext-processors-store";
import { TAG_NAME_MAP } from "../../../utils/custom-elements-registration/mod";

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
  const rotextProcessors = useRotextProcessorsStore()!;

  const [currentTab, setCurrentTab] = createSignal<Tab>(["preview"]);

  const [processResult, setProcessResult] = createSignal<
    RotextProcessResult | null
  >(null);
  const [isProcessorProviderReady, setIsProcessorProviderReady] = createSignal(
    false,
  );
  createEffect(
    on(
      [() => rotextProcessors.currentProvider, () => store.text],
      ([processorProvider, text]) => {
        setIsProcessorProviderReady(!!processorProvider);
        if (!processorProvider) return;
        const processor = processorProvider();

        setProcessResult(
          processor.process(text, {
            requiresLookupListRaw: true,
            tagNameMap: TAG_NAME_MAP,
          }),
        );
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
          <Show when={typeof processResult()?.parsingTimeMs === "number"}>
            <Badge>
              解析时间：{`${processResult()!.parsingTimeMs!.toFixed(3)}ms`}
            </Badge>
          </Show>
        </BadgeBar>
      </div>
    ),
    Preview: (
      <div class={`flex flex-col items-center ${opts.heightClass}`}>
        {
          /*
            原先本来只有一个接收 `processResult()` 的 `<Show />`，且
            `processResult()` 在初始情况之外是可以被设置回 null 的。（用来代表正
            在切换 rotext 处理器，fallback 到加载指示器。）

            原本好好的，不知怎么回事，不知什么时候起，突然就会在
            `setProcessResult(null)` 时报错称 “Attempting to access a stale
            value from <Show> that could possibly be undefined.”

            目前只好增加一层 `<Show />` 作为 workaround。

            先前在 `<Preview />` 里的错误警告处也遇到了这个问题，看来是出于相同
            的原因。
          */
        }
        <Show
          when={isProcessorProviderReady}
          fallback={
            <div
              class={`flex justify-center items-center h-full ${opts.widthClass}`}
            >
              <Loading />
            </div>
          }
        >
          <Show
            when={processResult()}
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
        </Show>
      </div>
    ),
  };
}
