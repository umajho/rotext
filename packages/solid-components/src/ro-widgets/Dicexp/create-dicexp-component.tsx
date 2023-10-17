import styles from "./DicexpPreview.module.scss";

import {
  Accessor,
  Component,
  createEffect,
  createSignal,
  Match,
  on,
  Show,
  Switch,
} from "solid-js";

import type { JSValue, Repr } from "dicexp";
import type {
  EvaluatingWorkerManager,
  EvaluationResultForWorker,
} from "@dicexp/evaluating-worker-manager";

import {
  ComputedColor,
  gray500,
  mouseDownNoDoubleClickToSelect,
} from "@rotext/web-utils";

import { createRoWidgetComponent } from "../../ro-widget-core/mod";

import { HorizontalRule, PinButton, WidgetContainer } from "../support";
import FaSolidDice from "./icons";
import { createStepsRepresentationComponent } from "./steps-representation";
import { createRoller, RuntimeLoadingStatus } from "./create-roller";
import { summarizeValue } from "./value-summary";

export interface DicexpResult {
  // runtimeInfo?: {
  //   evaluator?: SolutionSpecifier;
  //   topLevelScope?: SolutionSpecifier;
  //   seed?: string;
  // };
  result: ["value", JSValue] | ["value_summary", string] | "error" | [
    "error",
    string | Error,
  ];
  repr?: Repr;
  // statistics?: {
  //   timeConsumption?: { ms: number };
  // };
}

export interface SolutionSpecifier {
  name: string;
  version: string;
}

export interface DicexpEvaluatorProvider {
  default: () => Promise<EvaluatingWorkerManager<any>>;
  specified?: (
    evaluator: SolutionSpecifier,
    topLevelScope: SolutionSpecifier,
  ) => Promise<EvaluatingWorkerManager<any>>;
}

interface Properties {
  code: string;
  result: DicexpResult | null;
}

export interface CreateDicexpComponentOptions {
  backgroundColor: ComputedColor;
  widgetOwnerClass: string;
  innerNoAutoOpenClass?: string;
  evaluatorProvider?: DicexpEvaluatorProvider;
  Loading: Component;
  ErrorAlert: Component<{ error: Error; showsStack: boolean }>;
  StepsRepresentation: ReturnType<typeof createStepsRepresentationComponent>;
}

export function createDicexpComponent(
  opts: CreateDicexpComponentOptions,
): Component<Properties> {
  const {
    Loading,
    ErrorAlert,
    StepsRepresentation,
  } = opts;

  return (outerProps) => {
    // XXX: 暂时不考虑 reactivity
    const { rolling, resultDisplaying } = processProps(outerProps, opts);

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
      primeContentComponent: (props) => {
        return (
          <>
            <span
              class={`widget-prime-summary ${styles["dicexp-prime-content"]}`}
              style={{ cursor: props.cursor }}
              onClick={props.onToggleWidget}
              onMouseDown={mouseDownNoDoubleClickToSelect}
            >
              <FaSolidDice
                color="white"
                class={rolling?.isRolling() ? styles["animate-spin-400ms"] : ""}
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
                    <span class={styles["text-color-loading"]}>
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
                      <span class={resultSummary().textClass}>
                        {resultSummary().text}
                      </span>
                    </>
                  )}
                </Show>
              </span>
            </Show>
          </>
        );
      },
      widgetContainerComponent: WidgetContainer,
      widgetContentComponent: (props) => {
        return (
          <div class={styles["dicexp-widget-content"]}>
            <div class={styles["header"]}>
              <div class={styles["left-area"]}>
                <PinButton
                  displayMode={props.displayMode}
                  onClick={props.handlerForClickOnPinIcon}
                  onTouchEnd={props.handlerForTouchEndOnPinIcon}
                />
                <span>掷骰</span>
              </div>
            </div>
            <HorizontalRule />
            <div style={{ padding: "1rem" }}>
              <Switch>
                <Match when={resultDisplaying}>
                  {(resultDisplaying) => (
                    <>
                      <Show when={resultDisplaying().error()}>
                        {(resultError) => (
                          <ErrorAlert
                            error={resultError()}
                            showsStack={false}
                          />
                        )}
                      </Show>
                      <Show when={resultDisplaying().repr()}>
                        {(resultRepr) => (
                          <>
                            <div style={{ "padding-bottom": "0.5rem" }}>
                              <code>{outerProps.code}</code>
                              {" ➔"}
                            </div>
                            <StepsRepresentation repr={resultRepr()} />
                          </>
                        )}
                      </Show>
                    </>
                  )}
                </Match>
                <Match when={rolling?.isRolling()}>
                  <div class={styles["center-aligner"]}>
                    <Loading />
                  </div>
                </Match>
                <Match when={true}>
                  输入变更…
                </Match>
              </Switch>
            </div>
          </div>
        );
      },
    }, {
      widgetOwnerClass: opts.widgetOwnerClass,
      innerNoAutoOpenClass: opts.innerNoAutoOpenClass,
      openable: everRolled,
      autoOpenShouldCollapse: false,
      widgetBackgroundColor: () => opts.backgroundColor,
      maskTintColor: () => gray500,
    });

    return <>{component}</>;
  };
}

function processProps(
  outerProps: Properties,
  opts: CreateDicexpComponentOptions,
): {
  rolling?: {
    roll: (code: string) => Promise<void>;
    rtmLoadingStatus: Accessor<RuntimeLoadingStatus>;
    isRolling: Accessor<boolean>;
  };
  resultDisplaying?: {
    summary: () => { text: string; textClass?: string } | null;
    error: () => Error | null;
    repr: () => Repr | null;

    clear?: () => void;
  };
} {
  if (opts.evaluatorProvider && !outerProps.result) {
    const roller = createRoller({
      evaluatorProvider: opts.evaluatorProvider.default,
    });

    const [result, setResult] = //
      createSignal<EvaluationResultForWorker | null>(null);
    createEffect(on([roller.result], ([result]) => setResult(result)));

    return {
      rolling: {
        roll: roller.roll,
        rtmLoadingStatus: roller.rtmLoadingStatus,
        isRolling: roller.isRolling,
      },
      resultDisplaying: {
        summary: () => {
          const result_ = result();
          if (!result_) return null;

          if (result_[0] !== "ok") {
            return {
              text: "错误！",
              textClass: styles["text-color-error"]!,
            };
          }

          const summary = summarizeValue(result_[1]);
          if (summary === "too_complex") {
            return {
              text: "暂不支持显示的复杂值。",
              textClass: styles["text-color-warning"]!,
            };
          }
          return { text: summary[1] };
        },
        error: () => {
          const result_ = result();
          if (result_?.[0] === "error" /* && result_[1] !== "execute" */) {
            return result_[2];
          }
          return null;
        },
        repr: () => {
          const result_ = result();
          let repr: Repr | null = null;
          if (result_?.[0] === "ok") {
            repr = result_[2].representation;
          } else if (result_?.[0] === "error" && result_[1] === "execute") {
            repr = result_[3].representation;
          }
          return repr;
        },

        clear: () => setResult(null),
      },
    };
  } else if (outerProps.result) {
    return {
      resultDisplaying: {
        summary: () => {
          const resultSum = outerProps.result!.result;
          if (resultSum === "error" || resultSum[0] === "error") {
            return {
              text: "错误！",
              textClass: styles["text-color-error"]!,
            };
          } else if (resultSum[0] === "value") {
            const summary = summarizeValue(resultSum[1]);
            if (summary === "too_complex") {
              return ({
                text: "暂不支持显示的复杂值。",
                textClass: styles["text-color-warning"]!,
              });
            } else {
              return ({ text: summary[1] });
            }
          } else {
            resultSum[0] satisfies "value_summary";
            return ({
              text: resultSum[1],
            });
          }
        },
        error: () => {
          const resultSum = outerProps.result!.result;
          if (Array.isArray(resultSum) && resultSum[0] === "error") {
            // TODO: 应该让 ErrorAlert 本身支持 string
            return typeof resultSum[1] === "string"
              ? new Error(resultSum[1])
              : resultSum[1];
          }
          return null;
        },
        repr: () => outerProps.result?.repr,
      },
    };
  }

  return {};
}
