import { createSignal, JSX, lazy, Show, Suspense } from "solid-js";

import { Alert, Badge, BadgeBar, Loading, Tab, Tabs } from "../../ui";

import { EditorStore } from "../../../hooks/editor-store";

const Preview = lazy(() => import("./Preview/mod"));

export function createPreviewParts(
  store: EditorStore,
  opts: { widthClass: string; heightClass: string },
): {
  PreviewTopBar: JSX.Element;
  Preview: JSX.Element;
} {
  const [parsingTimeText, setParsingTimeText] = createSignal<string>(null);
  const [errParseInfo, setErrParseInfo] = createSignal<PreviewErrorInfo>(null);

  const handleThrowInParsing = (thrown: unknown) => {
    setErrParseInfo(extractPreviewErrorInfoFromThrown(thrown, "解析期间"));
  };

  return {
    PreviewTopBar: (
      <div class="flex h-full justify-between items-center">
        <Tabs>
          <Tab isActive={true}>预览</Tab>
        </Tabs>
        <BadgeBar>
          <Show when={parsingTimeText()}>
            <Badge>解析时间：{parsingTimeText()}</Badge>
          </Show>
        </BadgeBar>
      </div>
    ),
    Preview: (
      <div class={`flex flex-col items-center ${opts.heightClass}`}>
        <Show when={errParseInfo() !== null}>
          <Alert
            type="error"
            title={errParseInfo().title}
            message={errParseInfo().message}
            details={errParseInfo().details}
          />
        </Show>
        <Suspense
          fallback={
            <div
              class={`flex justify-center items-center h-full ${opts.widthClass}`}
            >
              <Loading />
            </div>
          }
        >
          <Preview
            store={store}
            class={`${opts.widthClass} px-4`}
            setParsingTimeText={setParsingTimeText}
            onThrowInParsing={handleThrowInParsing}
          />
        </Suspense>
      </div>
    ),
  };
}

interface PreviewErrorInfo {
  title: string;
  message: string;
  details?: string;
}

function extractPreviewErrorInfoFromThrown(
  thrown: unknown,
  when: string,
): PreviewErrorInfo {
  if (thrown instanceof Error) {
    return {
      title: when + "发生了错误",
      message: thrown.message,
      details: thrown.stack,
    };
  } else {
    return {
      title: when + "抛出了并非 `Error` 实例的值",
      message: `${thrown}`,
    };
  }
}
