import { Component, Index, Show } from "solid-js";

import { Badge, BadgeBar, Tab, Tabs } from "../../../components/ui/mod";

import { PreviewPartStore } from "./store";

const PreviewTopBar: Component<{ store: PreviewPartStore }> = (props) => {
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
      <BadgeBar>
        <Show
          when={typeof props.store.processResult?.parsingTimeMs === "number"}
        >
          <Badge>
            解析时间：{`${
              props.store.processResult!.parsingTimeMs!.toFixed(3)
            }ms`}
          </Badge>
        </Show>
      </BadgeBar>
    </div>
  );
};

export default PreviewTopBar;
