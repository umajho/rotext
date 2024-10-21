import { Component, createMemo, Index } from "solid-js";

import { Tab, Tabs } from "../../../components/ui/mod";

import { textEncoder } from "../../../utils/global";

import { PreviewPartStore } from "./store";

const PreviewTopBar: Component<{ store: PreviewPartStore }> = (props) => {
  const html = createMemo(() => {
    if (props.store.processResult?.html) {
      const html = props.store.processResult.html;
      return html.replace(/ data-block-id=".*?"/g, "");
    } else {
      return null;
    }
  });
  const infoText = createMemo(() => {
    const parts: string[] = [];
    if (typeof props.store.processResult?.parsingTimeMs === "number") {
      const timeMs = props.store.processResult.parsingTimeMs;
      parts.push(`解析${timeMs.toFixed(3)}毫秒`);
    }
    if (html()) {
      const byteCount = textEncoder.encode(html()!).length;
      parts.push(`${byteCount}字节`);
    }
    return parts.join(" | ");
  });

  return (
    <div class="flex h-full justify-between items-center">
      <Tabs>
        <Tab
          isActive={props.store.currentTab[0] === "preview"}
          onClick={() => props.store.currentTab = ["preview"]}
        >
          预览
        </Tab>
        <Tab
          isActive={props.store.currentTab[0] === "html"}
          onClick={() => props.store.currentTab = ["html"]}
        >
          HTML
        </Tab>
        <Index each={props.store.processResult?.extraInfos}>
          {(info, i) => (
            <Tab
              isActive={props.store.currentTab[0] === "extra" &&
                props.store.currentTab[1] === i}
              onClick={() => props.store.currentTab = ["extra", i]}
            >
              {info().name}
            </Tab>
          )}
        </Index>
      </Tabs>
      <div class="flex items-center gap-1">
        <span class="text-xs text-gray-500">{infoText()}</span>
      </div>
    </div>
  );
};

export default PreviewTopBar;
