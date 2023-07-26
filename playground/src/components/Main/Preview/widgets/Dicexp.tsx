import { Component, createSignal } from "solid-js";
import { customElement } from "solid-element";

import { createWidgetComponent } from "../../../../hooks/widgets";
import { PinButton, WidgetContainer } from "./support";
import { gray500 } from "../../../../utils/color-consts";
import {
  getComputedColor,
  getComputedCSSValueOfClass,
} from "../../../../utils/styles";
import FaSolidDice from "../../../icons";

const BACKGROUND_COLOR = getComputedColor(
  getComputedCSSValueOfClass("background-color", "previewer-background"),
);

interface Properties {
  code: string;
}

const Dicexp: Component<Properties> = (outerProps) => {
  const [openable, setOpenable] = createSignal(false);

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
              <FaSolidDice color="white" />
              <div class="w-2" />
              <span>{`[=${outerProps.code}]`}</span>
            </span>
            <span
              class={"inline-flex items-center h-[1.5rem] -mr-2 pl-1 pr-2 ml-1" +
                " border rounded-r-xl border-sky-700 bg-sky-700"}
            >
              试投
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
              hello {outerProps.code}
            </div>
          </div>
        );
      },
    },
    {
      openable,
      widgetBackgroundColor: () => BACKGROUND_COLOR,
      maskTintColor: () => gray500,
    },
  );

  return <>{component}</>;
};
export default Dicexp;

export function registerCustomElement(tag = "dicexp-preview") {
  customElement(tag, { code: null }, Dicexp);
}
