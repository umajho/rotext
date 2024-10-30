import { Component, createEffect, createSignal, on, Show } from "solid-js";

import * as Ankor from "ankor";

import { StyleProvider } from "@rolludejo/web-internal/shadow-root";
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
import { createPopperContent } from "./create-popper-content";
import { DicexpEvaluation } from "./evaluation";

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
  baseStyleProviders?: StyleProvider[];
  backgroundColor: ComputedColor;

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
          <span class="
            inline-flex max-w-full min-w-0 items-center h-[1.5rem] px-2 mx-2
            border border-solid rounded-xl border-sky-700 bg-sky-900 text-white
            font-sans">
            <span
              class="gap-1 w-auto min-w-0"
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
                class={rolling?.isRolling()
                  ? "animate-spin-400ms animate-[spin_400ms_linear_infinite]"
                  : ""}
              />
              <span class="
                font-mono text-[smaller]
                whitespace-nowrap overflow-hidden text-ellipsis">
                {`[=${outerProps.code}]`}
              </span>
            </span>
            <Show when={rolling?.roll || resultDisplaying?.summary()}>
              <span
                class="inline-flex min-w-max items-center h-[1.5rem]
                  -mr-2 pl-1 pr-2 ml-1 gap-2
                  border border-solid rounded-r-xl border-sky-700 bg-sky-700
                  font-bold"
                style={rolling
                  ? { cursor: "pointer", "user-select": "none" }
                  : {}}
                onClick={() => rolling?.roll(outerProps.code)}
              >
                <Show when={rolling}>
                  {(rolling) => (
                    <span class="text-gray-200">
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
      baseStyleProviders: opts.baseStyleProviders,
      openable: everRolled,
      autoOpenShouldCollapse: false,

      popperBackgroundColor: () => opts.backgroundColor,
      maskTintColor: () => gray500,
    });

    return <>{component}</>;
  };
}
