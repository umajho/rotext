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
import { customElement } from "solid-element";

import {
  EvaluatingWorkerManager,
  EvaluationResultForWorker,
  Repr,
} from "dicexp";

import { createRoWidgetComponent } from "@rotext/solid-components/internal";

import "./DicexpPreview.scss";

import { PinButton, WidgetContainer } from "./support";
import { gray500 } from "../../../../../utils/color-consts";
import {
  getComputedColor,
  getComputedCSSValueOfClass,
} from "@rotext/web-utils";
import FaSolidDice from "../../../../icons";
import { Loading } from "../../../../ui";
import { scopes } from "../../../../../stores/scopes";

import EvaluatingWorker from "../../../../../workers/dicexp-evaluator?worker";
import { ErrorAlert } from "../ui";
import { mouseDownNoDoubleClickToSelect } from "../../../../../utils/events";
import StepsRepresentation from "../../../../steps-representation";

const BACKGROUND_COLOR = getComputedColor(
  getComputedCSSValueOfClass("background-color", "tuan-background"),
)!;

interface Properties {
  code: string;
}

const DicexpPreview: Component<Properties> = (outerProps) => {
  const [loadingRuntime, setLoadingRuntime] = //
    createSignal<"short" | "long" | null>(null);
  const [rolling, setRolling] = createSignal(false);
  const [result, setResult] = //
    createSignal<EvaluationResultForWorker | null>(null);

  createEffect(on([() => outerProps.code], () => setResult(null)));

  const resultElement = createMemo(() => {
    const result_ = result();
    if (!result_) return null;
    if (result_[0] !== "ok") return <span class="text-red-500">错误！</span>;
    if (typeof result_[1] !== "number") {
      return <span class="text-red-500">暂不支持非数字结果！</span>;
    }
    return <>{String(result_[1])}</>;
  });

  const [everRolled, setEverRolled] = createSignal(false);
  createEffect(on([result], () => {
    if (!result() || everRolled()) return;
    setEverRolled(true);
  }));

  async function roll() {
    if (rolling()) return;
    setRolling(true);
    setResult(null);

    setLoadingRuntime("short");
    const cID = //
      setTimeout(() => loadingRuntime() && setLoadingRuntime("long"), 100);
    let dicexp: typeof import("dicexp") | undefined;
    try {
      dicexp = await import("dicexp");
    } catch (e) {
      const reason = (e instanceof Error) ? e.message : `e`;
      setResult(["error", "other", new Error(`加载运行时失败：${reason}`)]);
    }
    setLoadingRuntime(null);
    clearTimeout(cID);

    if (!dicexp) {
      setResult(null);
      setRolling(false);
      return;
    }

    const workerManager = await new Promise<
      EvaluatingWorkerManager<typeof scopes>
    >(
      (resolve) => {
        let resolved = false;
        const workerManager = new dicexp!.EvaluatingWorkerManager(
          () => new EvaluatingWorker(),
          (ready) => {
            if (resolved || !ready) return;
            resolve(workerManager);
            resolved = true;
          },
        );
      },
    );
    const result = await workerManager.evaluate(outerProps.code, {
      execute: { topLevelScopeName: "standard" },
    });

    workerManager.destroy();

    setResult(result);
    setRolling(false);
  }

  const component = createRoWidgetComponent(
    {
      primeContentComponent: (props) => {
        return (
          <>
            <span
              class="widget-prime-summary inline-flex items-center"
              style={{ cursor: props.cursor }}
              onClick={props.onToggleWidget}
              onMouseDown={mouseDownNoDoubleClickToSelect}
            >
              <FaSolidDice
                color="white"
                class={rolling() ? "animate-spin-400ms" : ""}
              />
              <span class="widget-prime-raw-text">
                {`[=${outerProps.code}]`}
              </span>
            </span>
            <span
              class="widget-prime-action"
              onClick={roll}
            >
              <span class="text-gray-200">
                {loadingRuntime() === "long"
                  ? "正在加载运行时…"
                  : (rolling() ? "正在试投…" : "试投")}
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
          <div class="flex flex-col">
            <div class="flex justify-between items-center px-2">
              <div class="flex items-center gap-2">
                <PinButton
                  displayMode={props.displayMode}
                  onClick={props.onClickOnPinIcon}
                  onTouchEnd={props.onTouchEndOnPinIcon}
                />
                <span>掷骰过程</span>
              </div>
            </div>
            <hr />
            <div class="p-4">
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
                <Match when={rolling()}>
                  <div class="flex justify-center items-center">
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
    },
    {
      openable: everRolled,
      autoOpenShouldCollapse: false,
      widgetBackgroundColor: () => BACKGROUND_COLOR,
      maskTintColor: () => gray500,
    },
  );

  return <>{component}</>;
};
export default DicexpPreview;

export function registerCustomElement(tag = "dicexp-preview") {
  customElement(tag, { code: "" }, DicexpPreview);
}
