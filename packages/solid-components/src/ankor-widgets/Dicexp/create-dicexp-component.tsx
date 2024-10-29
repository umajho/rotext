import { Component, createEffect, createSignal, on, Show } from "solid-js";

import * as Ankor from "ankor";

import {
  createStyleProviderFromCSSText,
  ShadowRootAttacher,
  StyleProvider,
} from "@rolludejo/web-internal/shadow-root";
import { ComputedColor } from "@rolludejo/web-internal/styling";

import type { EvaluatingWorkerManager } from "@dicexp/naive-evaluator-in-worker";

import { gray500, mouseDownNoDoubleClickToSelect } from "../../utils/mod";

import FaSolidDice from "./icons";
import { processProps } from "./props-for-create-dicexp-component";
import {
  ErrorAlertComponent,
  LoadingComponent,
  StepsRepresentationComponent,
} from "./external-components";
import {
  createPopperContent,
  styleProvider as styleProviderForPopperContent,
} from "./create-popper-content";
import { DicexpEvaluation } from "./evaluation";

import stylesForLabelContentSupplements from "./LabelContent.supplements.scss?inline";

const styleProviderForLabelContentSupplements = //
  createStyleProviderFromCSSText(stylesForLabelContentSupplements);

export interface DicexpEvaluatorProvider {
  default: () => Promise<EvaluatingWorkerManager>;
  // specified?: (
  //   evaluator: SolutionSpecifier,
  //   topLevelScope: SolutionSpecifier,
  // ) => Promise<EvaluatingWorkerManager>;
}

export interface Properties {
  code: string;
  evaluation: DicexpEvaluation | null;
}

export interface CreateDicexpComponentOptions {
  styleProviders: {
    forLabelContent: StyleProvider;
  };
  backgroundColor: ComputedColor;

  innerNoAutoOpenClass?: string;
  evaluatorProvider?: DicexpEvaluatorProvider;

  Loading: LoadingComponent;
  ErrorAlert: ErrorAlertComponent;
  StepsRepresentation: StepsRepresentationComponent;
}

export function createDicexpComponent(
  opts: CreateDicexpComponentOptions,
): Component<Properties> {
  const { Loading, ErrorAlert, StepsRepresentation } = opts;

  return (outerProps) => {
    // XXX: 暂时不考虑 reactivity
    const processedProperties = processProps(outerProps, opts);
    const { rolling, resultDisplaying } = processedProperties;

    createEffect(
      on([() => outerProps.code], () => resultDisplaying?.clear?.()),
    );

    const [everRolled, setEverRolled] = createSignal(false);
    if (resultDisplaying) {
      createEffect(on([resultDisplaying.summary], () => {
        if (!resultDisplaying.summary() || everRolled()) return;
        setEverRolled(true);
      }));
    }

    const component = Ankor.createWidgetComponent({
      LabelContent: (props) => {
        return (
          <ShadowRootAttacher
            hostStyle={{ display: "inline-flex", width: "100%" }}
            styleProviders={[
              styleProviderForLabelContentSupplements,
              opts.styleProviders.forLabelContent,
            ]}
          >
            <span class="widget-label-content">
              <span
                class="widget-label-summary"
                style={{
                  display: "inline-flex",
                  "place-items": "center",
                  cursor: props.cursor,
                }}
                onClick={() => props.onTogglePopper?.()}
                onMouseDown={mouseDownNoDoubleClickToSelect}
              >
                <FaSolidDice
                  color="white"
                  class={rolling?.isRolling() ? "animate-spin-400ms" : ""}
                />
                <span class="widget-label-raw-text">
                  {`[=${outerProps.code}]`}
                </span>
              </span>
              <Show when={rolling?.roll || resultDisplaying?.summary()}>
                <span
                  class={`widget-label-action`}
                  style={rolling
                    ? { cursor: "pointer", "user-select": "none" }
                    : {}}
                  onClick={() => rolling?.roll(outerProps.code)}
                >
                  <Show when={rolling}>
                    {(rolling) => (
                      <span class={"text-color-loading"}>
                        {rolling().rtmLoadingStatus() === "long"
                          ? "正在加载运行时…"
                          : (rolling().isRolling!() ? "正在试投…" : "试投")}
                      </span>
                    )}
                  </Show>
                  <Show when={resultDisplaying?.summary()}>
                    {(resultSummary) => (
                      <>
                        <span>➔</span>
                        <span
                          class={(
                            (level) => level && `text-color-${level}`
                          )(resultSummary().level)}
                        >
                          {resultSummary().text}
                        </span>
                      </>
                    )}
                  </Show>
                </span>
              </Show>
            </span>
          </ShadowRootAttacher>
        );
      },
      PopperContent: createPopperContent({
        code: () => outerProps.code,
        processedProperties,
        Loading,
        ErrorAlert,
        StepsRepresentation,
      }),
    }, {
      innerNoAutoOpenClass: opts.innerNoAutoOpenClass,
      openable: everRolled,
      autoOpenShouldCollapse: false,

      popperContentStyleProvider: styleProviderForPopperContent,
      popperBackgroundColor: () => opts.backgroundColor,
      maskTintColor: () => gray500,
    });

    return <>{component}</>;
  };
}
