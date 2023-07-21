import {
  Component,
  createMemo,
  createSignal,
  JSX,
  lazy,
  Show,
  Suspense,
} from "solid-js";

import { Alert, Badge, BadgeBar, Card, Loading, Tab, Tabs } from "../ui";

import * as examples from "@rotext/example-documentations";

import { createEditorStore, EditorStore } from "../../hooks/editor-store";

const MainCard: Component = () => {
  const store = createEditorStore(examples.introduction);

  const { EditorTopBar, Editor } = createEditorParts(store);
  const { PreviewTopBar, Preview } = createPreviewParts(store);

  return (
    <Card class="h-content">
      <div class="grid grid-cols-1 lg:grid-cols-2">
        <div class="max-lg:order-1">
          {EditorTopBar}
        </div>
        <div class="max-lg:order-3">
          {PreviewTopBar}
        </div>
        <div class="max-lg:order-2">
          {Editor}
        </div>
        <div class="max-lg:order-4">
          {Preview}
        </div>
      </div>
    </Card>
  );
};
export default MainCard;

const WIDTH_CLASS = "w-[80vw] lg:max-w-[35rem] lg:w-[45vw]";
const HEIGHT_CLASS =
  "h-[calc(50vh-8rem)] max-lg:!h-[calc(50dvh-8rem)] lg:h-[calc(100vh-16rem)]";

//==== Editor ====

const Editor = lazy(() => import("./Editor"));

const segmenter: Intl.Segmenter | null = (() => {
  if (window.Intl?.Segmenter) {
    return new Intl.Segmenter(undefined, { granularity: "grapheme" });
  }
  return null;
})();
const textEncoder = new TextEncoder();

function createEditorParts(store: EditorStore): {
  EditorTopBar: JSX.Element;
  Editor: JSX.Element;
} {
  const editorSizeClass = `${HEIGHT_CLASS} ${WIDTH_CLASS}`;

  const charCount = createMemo(() =>
    segmenter ? [...segmenter.segment(store.text)].length : null
  );
  const byteCount = createMemo(() => textEncoder.encode(store.text).length);
  const lineCount = createMemo(() => store.text.split("\n").length);
  const infoText = () =>
    [
      ...(charCount() !== null ? [`${charCount()}字`] : []),
      `${byteCount()}字节`,
      `${lineCount()}行`,
    ].join(" | ");

  return {
    EditorTopBar: (
      <div class="flex h-full justify-end items-center px-4">
        <span class="text-xs text-gray-500">{infoText()}</span>
      </div>
    ),
    Editor: (
      <Suspense
        fallback={
          <div class={`flex justify-center items-center ${editorSizeClass}`}>
            <Loading />
          </div>
        }
      >
        <Editor
          store={store}
          class={`${editorSizeClass} overflow-y-scroll`}
        />
      </Suspense>
    ),
  };
}

//==== Preview ====

const Preview = lazy(() => import("./Preview"));

function createPreviewParts(store: EditorStore): {
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
      <div class={`flex flex-col items-center ${HEIGHT_CLASS}`}>
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
              class={`flex justify-center items-center h-full ${WIDTH_CLASS}`}
            >
              <Loading />
            </div>
          }
        >
          <Preview
            store={store}
            class={`${WIDTH_CLASS} px-4`}
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
