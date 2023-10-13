import styles from "./DicexpPreview.module.scss";

import {
  Component,
  createEffect,
  createMemo,
  createSignal,
  Match,
  on,
  Show,
  Switch,
} from "solid-js";

import type { Repr } from "dicexp";

import {
  getComputedColor,
  getComputedCSSValueOfClass,
  gray500,
  mouseDownNoDoubleClickToSelect,
} from "@rotext/web-utils";

import { createRoWidgetComponent } from "../../ro-widget-core/mod";

import { PinButton, WidgetContainer } from "../support";
import FaSolidDice from "./icons";
import { createStepsRepresentationComponent } from "./steps-representation";
import { createRoller } from "./create-roller";

const BACKGROUND_COLOR = getComputedColor(
  getComputedCSSValueOfClass("background-color", "tuan-background"),
)!;

interface Properties {
  code: string;
}

export interface CreateDicexpComponentOptions {
  widgetOwnerClass: string;
  innerNoAutoOpenClass: string;
  dicexpImporter: () => Promise<typeof import("dicexp")>;
  EvaluatingWorker: new () => Worker;
  Loading: Component;
  ErrorAlert: Component<{ error: Error; showsStack: boolean }>;
  StepsRepresentation: ReturnType<typeof createStepsRepresentationComponent>;
}

export function createDicexpComponent(
  opts: CreateDicexpComponentOptions,
): Component<Properties> {
  const {
    dicexpImporter,
    EvaluatingWorker,
    Loading,
    ErrorAlert,
    StepsRepresentation,
  } = opts;

  return (outerProps) => {
    const { rtmLoadingStatus, isRolling, result, setResult, roll } = //
      createRoller({
        dicexpImporter,
        EvaluatingWorker,
      });

    createEffect(on([() => outerProps.code], () => setResult(null)));

    const resultElement = createMemo(() => {
      const result_ = result();
      if (!result_) return null;
      if (result_[0] !== "ok") {
        return <span class={styles["text-color-error"]}>错误！</span>;
      }
      if (typeof result_[1] !== "number") {
        return (
          <span class={styles["text-color-error"]}>暂不支持非数字结果！</span>
        );
      }
      return <>{String(result_[1])}</>;
    });

    const [everRolled, setEverRolled] = createSignal(false);
    createEffect(on([result], () => {
      if (!result() || everRolled()) return;
      setEverRolled(true);
    }));

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
                class={isRolling() ? styles["animate-spin-400ms"] : ""}
              />
              <span class="widget-prime-raw-text">
                {`[=${outerProps.code}]`}
              </span>
            </span>
            <span
              class="widget-prime-action"
              onClick={() => roll(outerProps.code)}
            >
              <span class={styles["text-color-loading"]}>
                {rtmLoadingStatus() === "long"
                  ? "正在加载运行时…"
                  : (isRolling() ? "正在试投…" : "试投")}
              </span>
              <Show when={resultElement()}>
                <span>➔</span>
                <span>{resultElement()}</span>
              </Show>
            </span>
          </>
        );
      },
      widgetContainerComponent: WidgetContainer,
      widgetContentComponent: (props) => {
        const resultError = (): Error | null => {
          const result_ = result();
          if (result_?.[0] === "error" /* && result_[1] !== "execute" */) {
            return result_[2];
          }
          return null;
        };
        const resultRepr = () => {
          const result_ = result();
          let repr: Repr | null = null;
          if (result_?.[0] === "ok") {
            repr = result_[2].representation;
          } else if (result_?.[0] === "error" && result_[1] === "execute") {
            repr = result_[3].representation;
          }
          return repr;
        };

        return (
          <div class={styles["dicexp-widget-content"]}>
            <div class={styles["header"]}>
              <div class={styles["left-area"]}>
                <PinButton
                  displayMode={props.displayMode}
                  onClick={props.onClickOnPinIcon}
                  onTouchEnd={props.onTouchEndOnPinIcon}
                />
                <span>掷骰过程</span>
              </div>
            </div>
            <hr />
            <div style={{ padding: "1rem" }}>
              <Switch>
                <Match when={result()}>
                  <Show
                    when={resultError()}
                  >
                    {(resultError) => (
                      <ErrorAlert error={resultError()} showsStack={false} />
                    )}
                  </Show>
                  <Show when={resultRepr()}>
                    <StepsRepresentation repr={resultRepr()} />
                  </Show>
                </Match>
                <Match when={isRolling()}>
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
      widgetBackgroundColor: () => BACKGROUND_COLOR,
      maskTintColor: () => gray500,
    });

    return <>{component}</>;
  };
}
