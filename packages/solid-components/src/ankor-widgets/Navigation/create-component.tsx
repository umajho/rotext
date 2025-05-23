import {
  batch,
  Component,
  createMemo,
  createSignal,
  onCleanup,
  onMount,
  Show,
} from "solid-js";

import * as Ankor from "ankor";

import { StyleProvider } from "@rolludejo/internal-web-shared/shadow-root";
import { ComputedColor } from "@rolludejo/internal-web-shared/styling";

import { gray500, mouseDownNoDoubleClickToSelect } from "../../utils/mod";

import {
  AnkorPopperHorizontalRule,
  AnkorPopperPinButton,
} from "../../ankor-support/mod";

import { createWatchableFromSignalGetter, Watchable } from "./hooks";

export interface Properties {
  address: string;
}

export type InnerRenderer = (
  address: Watchable<string>,
  opts: {
    /**
     * 更新导航文本（位于悬浮框右上角）。
     *
     * 比如，对于Wiki链接，第一个参数可能是：
     * - `[[页面#章节]]`，（加载中，或确定页面存在并且有对应章节时。）
     * - `[[页面]]`，（即使原本的地址包含章节，如果确定页面中没有对应章节，也
     *   会如此。）
     * - `创建[[页面]]`/`[[页面]]可能不存在`。（确定页面不存在时。）
     */
    updateNavigation: (
      opts: {
        text: string;
        action: (() => void) | null;
      },
    ) => void;
  },
) => {
  /**
   * 在调用时，是否已经准备好了自动打开。如果返回 `false`，代表没有准备好，不应
   * 该自动打开。
   *
   * 一般而言，评判是否准备好自动打开的标准是 “资源是否已经存在于本地，无需发起
   * 网络请求”。
   */
  isAutoOpenable: boolean;
  /**
   * 只会被调用一次。
   */
  render: (
    el: HTMLElement,
    opts: {
      widgetOwnerAgent: Ankor.WidgetOwnerAgent;

      onCleanup: (listener: () => void) => void;
    },
  ) => void;
};

export interface CreateNavigationComponentOptions {
  baseStyleProviders: StyleProvider[];
  classes: {
    forLabelWrapper: string;
    forNavigationAction: {
      enabled: string;
      disabled: string;
    };
  };
  backgroundColor: ComputedColor;

  label:
    | ["text", (address: string) => string]
    | ["slot"];
  innerPreviewRenderer: InnerRenderer;
}

export function createNavigationComponent(
  opts: CreateNavigationComponentOptions,
): Component<Properties> {
  return (outerProps) => {
    const Label = ((): Component<{ address: string }> => {
      switch (opts.label[0]) {
        case "text": {
          const [_, labelTextRenderer] = opts.label;
          return (props) => <>{labelTextRenderer(props.address)}</>;
        }
        case "slot":
          return () => <slot />;
        default:
          opts.label satisfies never;
          throw new Error("unreachable");
      }
    })();

    const addrW = createWatchableFromSignalGetter(() => outerProps.address);
    const [navText, setNavText] = createSignal<string | null>(null);
    const [navAction, setNavAction] = createSignal<(() => void) | null>(null);
    const renderer = opts.innerPreviewRenderer(addrW, {
      updateNavigation: (opts) => {
        batch(() => {
          setNavText(opts.text);
          setNavAction(() => opts.action);
        });
      },
    });

    const component = Ankor.createWidgetComponent({
      LabelContent: (props) => {
        let wrapperEl!: HTMLSpanElement;
        onMount(() => {
          // 不知为何，如果使用了 `slot`，`span` 上的 `onClick` 不会被触发；但如
          // 果没有使用 `slot`，一切就正常。为了让任何情况都能正常运作，这里手动
          // 添加事件监听器。
          wrapperEl.addEventListener("click", () => {
            props.onTogglePopper?.();
          });
        });

        return (
          <span
            ref={wrapperEl}
            class={opts.classes.forLabelWrapper}
            style={{
              cursor: props.cursor,
            }}
            onMouseDown={mouseDownNoDoubleClickToSelect}
          >
            <Label address={outerProps.address} />
          </span>
        );
      },
      PopperContent: (props) => {
        let refEl!: HTMLDivElement;

        const cleanupListeners: (() => void)[] = [];
        onMount(() => {
          renderer.render(refEl, {
            widgetOwnerAgent: props.widgetOwnerAgentGetter()!,
            onCleanup: (cb) => cleanupListeners.push(cb),
          });
        });
        onCleanup(() => cleanupListeners.forEach((listener) => listener()));

        const navActionClass = createMemo(() => {
          let cls = ["select-none"];
          const isDisabled = !navAction();
          const status = isDisabled ? "disabled" : "enabled";
          cls.push(opts.classes.forNavigationAction[status]);
          cls.push(isDisabled ? "cursor-default" : "cursor-pointer");
          return cls.join(" ");
        });

        return (
          <div class="flex flex-col">
            <div class="
              flex justify-between items-center px-2 leading-6
              text-gray-300 font-sans font-light">
              <AnkorPopperPinButton
                displayMode={props.displayMode}
                onClick={props.handlerForClickOnPinIcon}
              />
              <div style={{ width: "3rem" }} />
              <Show when={navText()}>
                {(navText) => (
                  <div
                    class={navActionClass()}
                    onClick={() => navAction?.()?.()}
                  >
                    {navText()}
                  </div>
                )}
              </Show>
            </div>
            <AnkorPopperHorizontalRule color="#fffa" />
            <div ref={refEl} />
          </div>
        );
      },
    }, {
      baseStyleProviders: opts.baseStyleProviders,
      autoOpenable: renderer.isAutoOpenable,

      popperBackgroundColor: () => opts.backgroundColor,
      maskTintColor: () => gray500,
    });

    return <>{component}</>;
  };
}
