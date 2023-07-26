import { Component, createEffect, createSignal, on, Show } from "solid-js";
import { customElement } from "solid-element";

import "./DicexpPreview.scss";

import { createWidgetComponent } from "../../../../hooks/widgets";
import { PinButton, WidgetContainer } from "./support";
import { gray500 } from "../../../../utils/color-consts";
import {
  getComputedColor,
  getComputedCSSValueOfClass,
} from "../../../../utils/styles";
import FaSolidDice from "../../../icons";
import { Loading } from "../../../ui";

const BACKGROUND_COLOR = getComputedColor(
  getComputedCSSValueOfClass("background-color", "previewer-background"),
);

interface Properties {
  code: string;
}

interface RollResult {
  finalValue: string;
  steps: string;
}

const DicexpPreview: Component<Properties> = (outerProps) => {
  const [rolling, setRolling] = createSignal(false);
  const [result, setResult] = createSignal<RollResult>();

  const [everRolled, setEverRolled] = createSignal(false);
  createEffect(on([result], () => {
    if (!result() || everRolled()) return;
    setEverRolled(true);
  }));

  function roll() {
    if (rolling()) return;
    setRolling(true);
    setResult(null);
    setTimeout(() => {
      setResult({
        finalValue: "42",
        steps:
          "Answer to the Ultimate Question of Life, the Universe, and Everything",
      });
      setRolling(false);
    }, 100 + Math.random() * 300);
  }

  const component = createWidgetComponent(
    {
      primeContentComponent: (props) => {
        return (
          <>
            <span
              class="inline-flex items-center"
              style={{ cursor: props.cursor }}
              onClick={props.onToggleWidget}
            >
              <FaSolidDice
                color="white"
                class={rolling() ? "animate-spin-400ms" : ""}
              />
              <div class="w-2" />
              <span>{`[=${outerProps.code}]`}</span>
            </span>
            <span
              class={"inline-flex items-center h-[1.5rem] -mr-2 pl-1 pr-2 ml-1" +
                " border rounded-r-xl border-sky-700 bg-sky-700" +
                " cursor-pointer select-none"}
              onClick={roll}
            >
              试投
              <Show when={result()}>
                {" ➔ "}
                {result().finalValue}
              </Show>
            </span>
          </>
        );
      },
      widgetContainerComponent: WidgetContainer,
      widgetContentComponent: (props) => {
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
              <Show
                when={result()}
                fallback={
                  <div class="flex justify-center items-center">
                    <Loading />
                  </div>
                }
              >
                {result().steps}
              </Show>
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
  customElement(tag, { code: null }, DicexpPreview);
}
