import styles from "./DicexpPreview.module.scss";

import { Component, createEffect, createSignal, on, Show } from "solid-js";

import type { JSValue, Repr } from "dicexp";
import type {
  EvaluatingWorkerManager,
} from "@dicexp/evaluating-worker-manager";

import {
  ComputedColor,
  gray500,
  mouseDownNoDoubleClickToSelect,
} from "@rotext/web-utils";

import { createRoWidgetComponent } from "../../ro-widget-core/mod";

import FaSolidDice from "./icons";
import { processProps } from "./props-for-create-dicexp-component";
import {
  ErrorAlert,
  Loading,
  StepsRepresentation,
} from "./external-components";
import { createWidgetContent } from "./create-widget-content";

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
  /**
   * 标记求值是在哪里进行的。
   *
   * TODO: 未来实现重建步骤的功能时，以数组类型的计算值存储统计，每项统计新增一个 location
   *       属性，以实现区分原先和重建后的统计内容。
   *      （如果原先本来就带步骤，那数组就只会有一项。）
   */
  location?: "local" | "server";
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

  Loading: Loading;
  ErrorAlert: ErrorAlert;
  StepsRepresentation: StepsRepresentation;
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
      WidgetContent: createWidgetContent({
        code: outerProps.code,
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
      widgetBackgroundColor: () => opts.backgroundColor,
      maskTintColor: () => gray500,
    });

    return <>{component}</>;
  };
}
