import { Component, createMemo } from "solid-js";
import { customElement } from "solid-element";

import * as Ankor from "ankor";

import { gray500 } from "@rotext/solid-components";
// FIXME!!: 应该正式由 `@rotext/solid-components` 或者其他哪里 export。
import { HorizontalRule, PinButton } from "@rotext/solid-components/internal";

import { styleProvider as styleProviderForPreflight } from "../../../styles/preflight";
import { styleProvider as styleProviderForTailwind } from "../../../styles/tailwind";

import { getBackgroundColor } from "../../utils";

import { ErrorMessage, getCallTypeTitle, Name } from "./shared";

function createInlineCallErrorComponent(): Component<{
  "call-type": "transclusion" | "extension" | "";
  "call-name": string;
  "error-type": string;
  "error-value": string | null;
}> {
  return (outerProps) => {
    const component = Ankor.createWidgetComponent({
      LabelContent: (props) => {
        const title = createMemo(() =>
          getCallTypeTitle(outerProps["call-type"])
        );

        return (
          <span
            onClick={props.onTogglePopper}
            class="inline-flex border border-red-500 border-dashed text-red-500"
            style={{
              cursor: props.cursor,
            }}
            onMouseDown={mouseDownNoDoubleClickToSelect}
          >
            {`${title()}「`}
            <Name name={outerProps["call-name"]} />
            {`」失败！`}
          </span>
        );
      },
      PopperContent: (props) => {
        return (
          <div class="flex flex-col font-sans text-gray-400">
            <div class="flex justify-between items-center px-2 leading-6">
              <div class="flex items-center gap-2">
                <PinButton
                  displayMode={props.displayMode}
                  onClick={props.handlerForClickOnPinIcon}
                />
                <span>错误信息</span>
              </div>
            </div>
            <HorizontalRule color="#fffa" />
            <div class="p-4">
              <ErrorMessage
                errorType={outerProps["error-type"]}
                errorValue={outerProps["error-value"]}
              />
            </div>
          </div>
        );
      },
    }, {
      baseStyleProviders: [styleProviderForPreflight, styleProviderForTailwind],
      autoOpenable: true,

      popperBackgroundColor: () => getBackgroundColor(),
      maskTintColor: () => gray500,
    });

    return <>{component}</>;
  };
}

/**
 * see: <https://stackoverflow.com/a/43321596>
 *
 * 复制自 `@rotext/solid-components` 中的 `src/utils/events.ts`。
 *
 * FIXME!!: 好像没用？（不止此处，其他挂件也是。）
 */
export function mouseDownNoDoubleClickToSelect(ev: MouseEvent) {
  if (ev.detail > 1) {
    ev.preventDefault();
  }
}

export function registerCustomElement(tag: string) {
  customElement(
    tag,
    { "call-type": "", "call-name": "", "error-type": "", "error-value": null },
    createInlineCallErrorComponent(),
  );
}
