import { Component, createEffect, on, onCleanup, onMount } from "solid-js";

import * as Ankor from "ankor";

import {
  createStyleProviderFromCSSText,
  StyleProvider,
} from "@rolludejo/web-internal/shadow-root";
import { ComputedColor } from "@rolludejo/web-internal/styling";

import { gray500, mouseDownNoDoubleClickToSelect } from "../../utils/mod";

import { HorizontalRule, PinButton } from "../support/mod";

import stylesForPopperContent from "./PopperContent.scss?inline";

const styleProviderForPopperContent = createStyleProviderFromCSSText(
  stylesForPopperContent,
);

export interface Properties {
  address: string;
}

export type InnerRenderer = (
  el: HTMLElement,
  address: string,
  opts: {
    onAddressChange: (listener: (addr: string) => void) => void;
    onCleanup: (listener: () => void) => void;
  },
) => void;

export interface CreateRefLinkComponentOptions {
  baseStyleProviders?: StyleProvider[];
  classes: {
    forLabelWrapper: string;
  };
  backgroundColor: ComputedColor;

  widgetOwnerClass: string;
  innerNoAutoOpenClass?: string;
  label:
    | ["text", (address: string) => string]
    | ["slot"];
  innerPreviewRenderer: InnerRenderer;
}

export function createRefLinkComponent(
  opts: CreateRefLinkComponentOptions,
): Component<Properties> {
  return (outerProps) => {
    const Label = ((): Component<{ address: string }> => {
      switch (opts.label[0]) {
        case "text": {
          const [_, labelTextRenderer] = opts.label;
          return (props) => <>{labelTextRenderer(props.address)}</>;
        }
        case "slot":
          return () => <slot name="content" />;
        default:
          opts.label satisfies never;
          throw new Error("unreachable");
      }
    })();

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
        let refElWrapper: { el: HTMLDivElement } = {} as any;

        setUpInnerRenderer({
          props: outerProps,
          refElWrapper,
          innerRenderer: opts.innerPreviewRenderer,
        });

        return (
          <div class="ref-link-widget-content">
            <div class="header">
              <PinButton
                displayMode={props.displayMode}
                onClick={props.handlerForClickOnPinIcon}
                onTouchEnd={props.handlerForTouchEndOnPinIcon}
              />
              <div style={{ width: "3rem" }} />
              <div>{outerProps.address}</div>
            </div>
            <HorizontalRule color="white" />
            <div ref={refElWrapper.el} />
          </div>
        );
      },
    }, {
      baseStyleProviders: opts.baseStyleProviders,
      widgetOwnerClass: opts.widgetOwnerClass,
      innerNoAutoOpenClass: opts.innerNoAutoOpenClass,

      popperContentStyleProvider: styleProviderForPopperContent,
      popperBackgroundColor: () => opts.backgroundColor,
      maskTintColor: () => gray500,
    });

    return <>{component}</>;
  };
}

function setUpInnerRenderer(
  opts: {
    props: { address: string };
    refElWrapper: { el: HTMLDivElement };
    innerRenderer: InnerRenderer;
  },
) {
  onMount(() => {
    const changeListeners: ((addr: string) => void)[] = [];
    const cleanupListeners: (() => void)[] = [];
    opts.innerRenderer(opts.refElWrapper.el, opts.props.address, {
      onAddressChange: (listener) => changeListeners.push(listener),
      onCleanup: (listener) => cleanupListeners.push(listener),
    });
    createEffect(on(
      [() => opts.props.address],
      ([address]) => changeListeners.forEach((listener) => listener(address)),
      { defer: true },
    ));
    onCleanup(() => cleanupListeners.forEach((listener) => listener()));
  });
}
