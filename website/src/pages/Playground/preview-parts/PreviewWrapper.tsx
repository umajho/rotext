import {
  Component,
  createEffect,
  createSignal,
  lazy,
  Match,
  on,
  Show,
  Switch,
} from "solid-js";

import { Loading } from "../../../components/ui/mod";
import { useRotextProcessorsStore } from "../../../contexts/rotext-processors-store";
import { TAG_NAME_MAP } from "../../../utils/custom-elements-registration/mod";

import { EditorStore } from "../editor-store";

import { PreviewPartStore } from "./store";
import { formatHTML } from "../../../utils/html-formatting";

const Preview = lazy(() => import("./Preview/mod"));

type Tab =
  | ["preview"]
  | ["html"]
  | [type: "extra", index: number];

const PreviewWrapper: Component<{
  store: PreviewPartStore;
  widthClass: string;
  heightClass: string;
  editorStore: EditorStore;
}> = (props) => {
  const rotextProcessors = useRotextProcessorsStore()!;

  const [isProcessorProviderReady, setIsProcessorProviderReady] = createSignal(
    false,
  );
  createEffect(
    on(
      [() => rotextProcessors.currentProvider, () => props.editorStore.text],
      ([processorProvider, text]) => {
        setIsProcessorProviderReady(!!processorProvider);
        if (!processorProvider) return;
        const processor = processorProvider();

        props.store.processResult = processor.process(text, {
          requiresLookupListRaw: true,
          tagNameMap: TAG_NAME_MAP,
        });
      },
    ),
  );

  return (
    <div class={`flex flex-col items-center ${props.heightClass}`}>
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
            class={`flex justify-center items-center h-full ${props.widthClass}`}
          >
            <Loading />
          </div>
        }
      >
        <Show
          when={props.store.processResult}
        >
          {(processResult) => (
            <>
              <Preview
                store={props.editorStore}
                processResult={processResult()}
                class={`${props.widthClass} px-4`}
                hidden={props.store.currentTab[0] !== "preview"}
              />
              <Switch>
                <Match
                  when={props.store.currentTab[0] === "html" &&
                    processResult().html}
                >
                  {
                    /*
                      由于不明原因，pre 默认的 `whitespace: pre` 会导致其内容与
                      编辑器部分重叠起来，所以手动设为 `whitespace: pre-wrap`。
                    */
                  }
                  <pre class="overflow-scroll whitespace-pre-wrap">
                    <code>{formatHTML(processResult().html!)}</code>
                  </pre>
                </Match>
                <Match when={props.store.currentTab[0] === "extra"}>
                  <pre class="overflow-scroll whitespace-pre-wrap">
                    {processResult().extraInfos[(props.store.currentTab as Extract<Tab, {0:"extra"}>)[1]]?.content}
                  </pre>
                </Match>
              </Switch>
            </>
          )}
        </Show>
      </Show>
    </div>
  );
};

export default PreviewWrapper;
