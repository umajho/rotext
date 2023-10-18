import styles from "./DicexpPreview.module.scss";

import {
  Accessor,
  Component,
  createEffect,
  createMemo,
  createSignal,
  Match,
  on,
  Show,
  Switch,
} from "solid-js";

import type { ExecutionAppendix, JSValue, Repr } from "dicexp";
import type {
  EvaluatingWorkerManager,
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

export interface DicexpEvaluation {
  /**
   * 求值环境。在重建求值结果时需要用到。
   */
  environment?: [
    /**
     * 求值器的名称。版本是名称的一部分。
     *
     * 如：`"$@0.4.1"` 或者等价的 `"dicexp@0.4.1"`
     */
    evaluatorName: string,
    /**
     * 求值器运行时的信息。求值器应该保证在相同的信息下，求值的结果（包括步骤）总是相同。
     *
     * 比如，对于 dicexp@0.4.1 而言，要满足上述条件，信息要包括：随机数生成方案名、种子数、顶部作用域的路径。
     *
     * 如：`"{r:42,s:"0.4.0"}"`，或者等价的 `"{r:["xorshift7",42],s:["@dicexp/builtins@0.4.0","./essence/standard-soceps","standard"]}"`。
     * （其中，“r” 代表 “Rng (Random number generator)”，“s” 代表 “top level Scope”。）
     */
    runtimeInfo: string,
  ];
  result: ["value", JSValue] | ["value_summary", string] | "error" | [
    "error",
    string | Error,
  ];
  repr?: Repr;
  statistics?: {
    timeConsumption?: { ms: number };
  };
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

export interface Properties {
  code: string;
  evaluation: DicexpEvaluation | null;
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
        const [showsMoreInExtraInfo, setShowsMoreInExtraInfo] = //
          createSignal(false);

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
            <div style={{ padding: "0.5rem 0.5rem 0 0.5rem" }}>
              <div style={{ padding: "0 0.5rem 0 0.5rem" }}>
                <Switch>
                  <Match when={resultDisplaying?.summary()}>
                    <Show when={resultDisplaying!.error()}>
                      {(resultError) => (
                        <ErrorAlert
                          error={resultError()}
                          showsStack={false}
                        />
                      )}
                    </Show>
                    <Show when={resultDisplaying!.repr()}>
                      {(resultRepr) => (
                        <>
                          <div>
                            <code class={styles["code"]}>
                              {outerProps.code}
                            </code>
                            {" ➔"}
                          </div>
                          <StepsRepresentation repr={resultRepr()} />
                        </>
                      )}
                    </Show>
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
              <Show
                when={resultDisplaying?.statistics() ||
                  resultDisplaying?.environment()}
                fallback={<div style={{ height: "0.5rem" }} />}
              >
                <div class={styles["extra-info"]}>
                  <div>
                    <Show
                      when={resultDisplaying!.statistics()?.timeConsumption}
                    >
                      {(timeConsumption) => `耗时≈${timeConsumption().ms}ms`}
                    </Show>
                    <Show
                      when={!showsMoreInExtraInfo() &&
                        resultDisplaying!.environment()}
                    >
                      {" "}
                      <span
                        class={styles["more"]}
                        onClick={() => setShowsMoreInExtraInfo(true)}
                      >
                        …
                      </span>
                    </Show>
                  </div>
                  <Show
                    when={showsMoreInExtraInfo() &&
                      resultDisplaying!.environment()}
                  >
                    {(environment) => (
                      <>
                        <div>{`求值器=${environment()[0]}`}</div>
                        <div>{`运行时信息=${environment()[1]}`}</div>
                      </>
                    )}
                  </Show>
                </div>
              </Show>
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
    environment: () =>
      | NonNullable<DicexpEvaluation["environment"]>
      | null;
    statistics: () => NonNullable<DicexpEvaluation["statistics"]> | null;

    clear?: () => void;
  };
} {
  if (opts.evaluatorProvider && !outerProps.evaluation) {
    const roller = createRoller({
      evaluatorProvider: opts.evaluatorProvider.default,
    });

    const appendix = createMemo((): ExecutionAppendix | null => {
      const result = roller.result();
      if (result?.[0] === "ok") {
        return result[2];
      } else if (result?.[0] === "error" && result[1] === "execute") {
        return result[3];
      }
      return null;
    });

    return {
      rolling: {
        roll: roller.roll,
        rtmLoadingStatus: roller.rtmLoadingStatus,
        isRolling: roller.isRolling,
      },
      resultDisplaying: {
        summary: () => {
          const result = roller.result();
          if (!result) return null;

          if (result[0] !== "ok") {
            return {
              text: "错误！",
              textClass: styles["text-color-error"]!,
            };
          }

          const summary = summarizeValue(result[1]);
          if (summary === "too_complex") {
            return {
              text: "暂不支持显示的复杂值。",
              textClass: styles["text-color-warning"]!,
            };
          }
          return { text: summary[1] };
        },
        error: () => {
          const result = roller.result();
          if (result?.[0] === "error" /* && result_[1] !== "execute" */) {
            return result[2];
          }
          return null;
        },
        repr: () => appendix()?.representation ?? null,
        environment: roller.environment,
        statistics: () => appendix()?.statistics ?? null,

        clear: roller.clear,
      },
    };
  } else if (outerProps.evaluation) {
    return {
      resultDisplaying: {
        summary: () => {
          const resultSum = outerProps.evaluation!.result;
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
          const resultSum = outerProps.evaluation!.result;
          if (Array.isArray(resultSum) && resultSum[0] === "error") {
            // TODO: 应该让 ErrorAlert 本身支持 string
            return typeof resultSum[1] === "string"
              ? new Error(resultSum[1])
              : resultSum[1];
          }
          return null;
        },
        repr: () => outerProps.evaluation?.repr ?? null,
        environment: () => outerProps.evaluation?.environment ?? null,
        statistics: () => outerProps.evaluation?.statistics ?? null,
      },
    };
  }

  return {};
}
