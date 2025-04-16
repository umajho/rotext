import { Component, createMemo, Match, Switch } from "solid-js";
import { customElement } from "solid-element";
import { OcLinkexternal3 } from "solid-icons/oc";

import * as Ankor from "ankor";

import {
  AnkorPopperHorizontalRule,
  AnkorPopperPinButton,
  gray500,
  mouseDownNoDoubleClickToSelect,
} from "@rotext/solid-components";

import { styleProvider as styleProviderForPreflight } from "../../../styles/preflight";
import { styleProvider as styleProviderForTailwind } from "../../../styles/tailwind";

import { getBackgroundColor } from "../../utils";

function createExternalLinkComponent(): Component<{
  "address": string;
}> {
  return (outerProps) => {
    const url = createMemo(() =>
      URL.parse(outerProps.address) ??
        URL.parse("https://" + outerProps.address)
    );
    const urlSummary = createMemo(() => {
      const url_ = url();
      if (!url_) return "未知";
      const url2 = new URL(url_);
      const hasPath = url2.pathname !== "/";
      url2.pathname = "/";
      return `${url2}${hasPath ? "..." : ""}`;
    });
    const schemaSafetyLevel = createMemo(
      (): "safe" | "unknown" | "dangerous" => {
        const url_ = url();
        if (!url_) return "unknown";
        const schema = url_.protocol.slice(0, -1);
        if (["http", "https"].includes(schema)) return "safe";
        if (["javascript", "data", "blob"].includes(schema)) return "dangerous";
        return "unknown";
      },
    );

    const component = Ankor.createWidgetComponent({
      LabelContent: (props) => {
        return (
          <span
            onClick={props.onTogglePopper}
            class="text-blue-700 underline"
            style={{
              cursor: props.cursor,
            }}
            onMouseDown={mouseDownNoDoubleClickToSelect}
          >
            <OcLinkexternal3 class="inline" />
            {urlSummary()}
          </span>
        );
      },
      PopperContent: (props) => {
        return (
          <div class="flex flex-col font-sans text-gray-400">
            <div class="flex justify-between items-center px-2 leading-6">
              <div class="flex items-center gap-2">
                <AnkorPopperPinButton
                  displayMode={props.displayMode}
                  onClick={props.handlerForClickOnPinIcon}
                />
                <span>外部链接</span>
              </div>
            </div>
            <AnkorPopperHorizontalRule color="#fffa" />
            <div class="p-4">
              <Switch>
                <Match when={schemaSafetyLevel() === "safe"}>
                  <p>本站无法确保该链接的安全性，请注意安全。</p>
                  <p class="py-2">
                    以下是该链接的完整地址。点击以访问：
                  </p>
                  <p>
                    <a
                      href={outerProps.address}
                      target="_blank"
                      rel="noreferrer"
                      class="text-blue-700 underline"
                    >
                      {outerProps.address}
                    </a>
                  </p>
                </Match>
                <Match when={schemaSafetyLevel() === "unknown"}>
                  <p>
                    本站无法确保该链接的安全性，请注意安全，在使用前确保理解其原理。
                  </p>
                  <p class="py-2">
                    以下是该链接的完整地址，请手动复制：
                  </p>
                  <p class="select-all">{outerProps.address}</p>
                </Match>
                <Match when={schemaSafetyLevel() === "dangerous"}>
                  <p>
                    该链接<span class="text-red-500">
                      十分危险，可能用于执行恶意代码
                    </span>，请不要在不理解的情况下使用！
                  </p>
                  <p class="py-2">
                    以下是该链接的完整地址，请手动复制：
                  </p>
                  <p class="select-all">{outerProps.address}</p>
                </Match>
              </Switch>
            </div>
          </div>
        );
      },
    }, {
      baseStyleProviders: [styleProviderForPreflight, styleProviderForTailwind],
      autoOpenable: false,

      popperBackgroundColor: () => getBackgroundColor(),
      maskTintColor: () => gray500,
    });

    return <>{component}</>;
  };
}

export function registerCustomElement(tag: string) {
  customElement(
    tag,
    { "address": "" },
    createExternalLinkComponent(),
  );
}
