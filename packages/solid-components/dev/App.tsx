import "./App.css";
import styles from "./App.module.css";

import { type Component, onMount } from "solid-js";

import { registerCustomElementForStepRepresentations } from "@dicexp/solid-components";

import {
  getComputedColor,
  getComputedCSSValueOfClass,
} from "@rotext/web-utils";

import {
  registerCustomElementForRoWidgetDicexp,
  registerCustomElementForRoWidgetRefLink,
  registerRoWidgetOwner,
  withDefaultDicexpStyle,
  withDefaultRefLinkStyle,
} from "../src/index";

import {
  createWorkerByImportURLs,
  EvaluatingWorkerManager,
} from "@dicexp/evaluating-worker-manager";
import dicexpImportURL from "dicexp/essence/for-worker?url";
import scopesImportURL from "@dicexp/builtins/essence/standard-scopes?url";

const WIDGET_OWNER_CLASS = "widget-owner";

const BACKGROUND_COLOR = getComputedColor(
  getComputedCSSValueOfClass("background-color", styles["App"]!),
)!;

registerCustomElementForRoWidgetRefLink("ro-widget-ref-link", {
  withStyle: withDefaultRefLinkStyle,
  backgroundColor: BACKGROUND_COLOR,
  widgetOwnerClass: WIDGET_OWNER_CLASS,
  refContentRenderer: (el, addr, onAddressChange) => {
    el.innerText = JSON.stringify(addr);
    onAddressChange(() => el.innerText = JSON.stringify(addr));
  },
});
registerCustomElementForStepRepresentations("steps-representation");
registerCustomElementForRoWidgetDicexp("ro-widget-dicexp", {
  withStyle: withDefaultDicexpStyle,
  backgroundColor: BACKGROUND_COLOR,
  widgetOwnerClass: WIDGET_OWNER_CLASS,
  evaluatorProvider: () => {
    const createWorker = () =>
      createWorkerByImportURLs(
        (new URL(dicexpImportURL, window.location.href)).href,
        (new URL(scopesImportURL, window.location.href)).href,
      );
    return new Promise(
      (resolve) => {
        let resolved = false;
        const workerManager = new EvaluatingWorkerManager(
          createWorker,
          (ready) => {
            if (resolved || !ready) return;
            resolve(workerManager);
            resolved = true;
          },
        );
      },
    );
  },
  Loading: () => "loading…",
  ErrorAlert: () => "error!",
  tagNameForStepsRepresentation: "steps-representation",
});

declare module "solid-js" {
  namespace JSX {
    interface IntrinsicElements {
      "ro-widget-ref-link": { address: string };
      "ro-widget-dicexp": { code: string };
    }
  }
}

const App: Component = () => {
  let widgetAnchorLeftEl!: HTMLDivElement,
    widgetAnchorRightEl!: HTMLDivElement,
    widgetAnchorLeftInnerEl!: HTMLDivElement;

  onMount(() => {
    [widgetAnchorLeftEl, widgetAnchorRightEl, widgetAnchorLeftInnerEl].forEach(
      (AnchorEl) => {
        const ownerEl: HTMLElement = //
          AnchorEl.closest("." + WIDGET_OWNER_CLASS)!;
        const controller = registerRoWidgetOwner(
          ownerEl,
          { widgetAnchorElement: AnchorEl, level: 1 },
        );
        const o = new ResizeObserver(() => controller.nofityLayoutChange());
        o.observe(ownerEl);
      },
    );
  });

  return (
    <div class={styles.App}>
      <div class="grid grid-cols-2 bg-slate-950">
        <div class={`${WIDGET_OWNER_CLASS}`}>
          <div ref={widgetAnchorLeftEl} class="relative z-10" />
          <div class="grid grid-rows-3 h-screen">
            <ro-widget-dicexp code="d100" />
            <div class="p-4 overflow-y-scroll">
              <div class="h-screen bg-slate-900">
                <div
                  class={`${WIDGET_OWNER_CLASS} bg-slate-800`}
                >
                  <div ref={widgetAnchorLeftInnerEl} class="relative z-10" />
                  <ro-widget-dicexp code="d100" />
                  <ro-widget-dicexp code="d100" />
                  <div class="h-[50vh]" />
                  <ro-widget-dicexp code="d100" />
                  <ro-widget-dicexp code="d100" />
                </div>
              </div>
            </div>
            <div>
              <ro-widget-dicexp code="d100" />
            </div>
          </div>
        </div>
        <div class={`${WIDGET_OWNER_CLASS} bg-stone-950`}>
          <div ref={widgetAnchorRightEl} class="relative z-10" />
          <div class="h-[50vh]" />
          <ro-widget-ref-link address="TP.foo" />
          <ro-widget-dicexp code="d100" />
          <ro-widget-dicexp code="d100" />
        </div>
      </div>
    </div>
  );
};

export default App;
