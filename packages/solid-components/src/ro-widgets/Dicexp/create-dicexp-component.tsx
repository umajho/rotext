import { Component, createEffect, createSignal, on, Show } from "solid-js";

import type {
  EvaluatingWorkerManager,
} from "@dicexp/evaluating-worker-manager";

import {
  ComputedColor,
  createStyleProviderFromCSSText,
  gray500,
  mouseDownNoDoubleClickToSelect,
  StyleProvider,
} from "@rotext/web-utils";

import { createRoWidgetComponent } from "../../ro-widget-core/mod";
import { ShadowRootAttacher } from "../../internal/mod";

import FaSolidDice from "./icons";
import { processProps } from "./props-for-create-dicexp-component";
import {
  ErrorAlertComponent,
  LoadingComponent,
  StepsRepresentationComponent,
} from "./external-components";
import {
  createWidgetContent,
  styleProvider as styleProviderForWidgetContent,
} from "./create-widget-content";
import { DicexpEvaluation } from "./evaluation";

import stylesForPrimeContentSupplements from "./PrimeContent.supplements.scss?inline";

const styleProviderForPrimeContentSupplements = //
  createStyleProviderFromCSSText(stylesForPrimeContentSupplements);

export interface DicexpEvaluatorProvider {
  default: () => Promise<EvaluatingWorkerManager<any>>;
  // specified?: (
  //   evaluator: SolutionSpecifier,
  //   topLevelScope: SolutionSpecifier,
  // ) => Promise<EvaluatingWorkerManager<any>>;
}

export interface Properties {
  code: string;
  evaluation: DicexpEvaluation | null;
}

export interface CreateDicexpComponentOptions {
  styleProviders: {
    forPrimeContent: StyleProvider;
  };
  backgroundColor: ComputedColor;

  widgetOwnerClass: string;
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

    const component = createRoWidgetComponent({
      PrimeContent: (props) => {
        return (
          <ShadowRootAttacher
            hostStyle={{ display: "inline-flex", width: "100%" }}
            styleProviders={[
              styleProviderForPrimeContentSupplements,
              opts.styleProviders.forPrimeContent,
            ]}
          >
            <span class="widget-prime-content">
              <span
                class="widget-prime-summary"
                style={{
                  display: "inline-flex",
                  "place-items": "center",
                  cursor: props.cursor,
                }}
                onClick={() => props.onToggleWidget?.()}
                onMouseDown={mouseDownNoDoubleClickToSelect}
              >
                <FaSolidDice
                  color="white"
                  class={rolling?.isRolling() ? "animate-spin-400ms" : ""}
                />
                <span class="widget-prime-raw-text">
                  {`[=${outerProps.code}]`}
                </span>
              </span>
              <Show when={rolling?.roll || resultDisplaying?.summary()}>
                <span
                  class={`widget-prime-action`}
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
      WidgetContent: createWidgetContent({
        code: () => outerProps.code,
        processedProperties,
        Loading,
        ErrorAlert,
        StepsRepresentation,
      }),
    }, {
      widgetOwnerClass: opts.widgetOwnerClass,
      innerNoAutoOpenClass: opts.innerNoAutoOpenClass,
      openable: everRolled,
      autoOpenShouldCollapse: false,

      widgetContentStyleProvider: styleProviderForWidgetContent,
      widgetBackgroundColor: () => opts.backgroundColor,
      maskTintColor: () => gray500,
    });

    return <>{component}</>;
  };
}
