import { Component, createSignal, lazy, Show, Suspense } from "solid-js";

import { Alert, Badge, BadgeBar, Card, Loading, Tab, Tabs } from "../ui";

import { EditorStore } from "../../hooks/editor-store";

const Preview = lazy(() => import("./Preview"));

const ViewerCard: Component<{ store: EditorStore }> = (props) => {
  const previewSizeClass = "h-full max-h-[25vh] lg:max-h-none";

  const [parsingTimeText, setParsingTimeText] = createSignal<string>(null);
  const [errParseInfo, setErrParseInfo] = createSignal<ErrorInfo>(null);

  const handleThrowInParsing = (thrown: unknown) => {
    setErrParseInfo(extractInfoFromThrown(thrown, "解析期间"));
  };

  return (
    <Card class="w-full max-w-[48rem] lg:w-[36rem] lg:max-h-[80vh]">
      <Show when={errParseInfo() !== null}>
        <Alert
          type="error"
          title={errParseInfo().title}
          message={errParseInfo().message}
          details={errParseInfo().details}
        />
      </Show>
      <div class="flex justify-between items-center">
        <Tabs>
          <Tab isActive={true}>预览</Tab>
        </Tabs>
        <BadgeBar>
          <Show when={parsingTimeText()}>
            <Badge>解析时间：{parsingTimeText()}</Badge>
          </Show>
        </BadgeBar>
      </div>
      <Suspense
        fallback={
          <div class={`flex justify-center items-center ${previewSizeClass}`}>
            <Loading />
          </div>
        }
      >
        <Preview
          store={props.store}
          class={previewSizeClass}
          setParsingTimeText={setParsingTimeText}
          onThrowInParsing={handleThrowInParsing}
        />
      </Suspense>
    </Card>
  );
};
export default ViewerCard;

interface ErrorInfo {
  title: string;
  message: string;
  details?: string;
}

function extractInfoFromThrown(thrown: unknown, when: string): ErrorInfo {
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
