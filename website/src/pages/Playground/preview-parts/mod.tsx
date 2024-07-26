import { createEffect, createSignal, JSX, lazy, on, Show } from "solid-js";

import {
  Badge,
  BadgeBar,
  Loading,
  Tab,
  Tabs,
} from "../../../components/ui/mod";

import { EditorStore } from "../../../hooks/editor-store";
import { OldRotextProcessor } from "../../../processors/old";
import { RotextProcessResult } from "../../../processors/mod";

const Preview = lazy(() => import("./Preview/mod"));

export function createPreviewParts(
  store: EditorStore,
  opts: { widthClass: string; heightClass: string },
): {
  PreviewTopBar: JSX.Element;
  Preview: JSX.Element;
} {
  const rotextProcessor = new OldRotextProcessor();

  const [processResult, setProcessResult] = createSignal<
    RotextProcessResult | null
  >(null);
  createEffect(on([() => store.text], ([text]) => {
    setProcessResult(rotextProcessor.process(text));
  }));

  return {
    PreviewTopBar: (
      <div class="flex h-full justify-between items-center">
        <Tabs>
          <Tab isActive={true}>预览</Tab>
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
            <Preview
              store={store}
              processResult={processResult()}
              class={`${opts.widthClass} px-4`}
            />
          )}
        </Show>
      </div>
    ),
  };
}
