import "./App.css";
import styles from "./App.module.css";

import { type Component, createSignal, Index, onMount } from "solid-js";

import { registerCustomElementForStepsRepresentation } from "@dicexp/solid-components";

import {
  getComputedColor,
  getComputedCSSValueOfClass,
} from "@rotext/web-utils";

import {
  getDefaultDicexpStyleProviders,
  getDefaultRefLinkStyleProviders,
  registerCustomElementForRoWidgetDicexp,
  registerCustomElementForRoWidgetRefLink,
  registerRoWidgetOwner,
} from "../internal";

import {
  createWorkerByImportURLs,
  EvaluatingWorkerManager,
} from "@dicexp/evaluating-worker-manager";
import dicexpImportURL from "dicexp/essence/for-worker?url";
import scopesImportURL from "@dicexp/builtins/essence/standard-scopes?url";
import { DicexpEvaluation } from "src/ro-widgets/Dicexp/create-dicexp-component";

const WIDGET_OWNER_CLASS = "widget-owner";

const BACKGROUND_COLOR = getComputedColor(
  getComputedCSSValueOfClass("background-color", styles["App"]!),
)!;

registerCustomElementForRoWidgetRefLink("ro-widget-ref-link", {
  styleProviders: getDefaultRefLinkStyleProviders(),
  backgroundColor: BACKGROUND_COLOR,
  widgetOwnerClass: WIDGET_OWNER_CLASS,
  refContentRenderer: (el, addr, onAddressChange) => {
    el.innerText = JSON.stringify(addr);
    el.style.color = "white";
    onAddressChange(() => el.innerText = JSON.stringify(addr));
  },
});
registerCustomElementForStepsRepresentation("steps-representation");
registerCustomElementForRoWidgetDicexp("ro-widget-dicexp", {
  styleProviders: getDefaultDicexpStyleProviders(),
  backgroundColor: BACKGROUND_COLOR,
  widgetOwnerClass: WIDGET_OWNER_CLASS,
  evaluatorProvider: {
    default: () => {
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
  },
  Loading: () => "loading…",
  ErrorAlert: () => "error!",
  tagNameForStepsRepresentation: "steps-representation",
});
registerCustomElementForRoWidgetDicexp("ro-widget-dicexp-no-runtime", {
  styleProviders: getDefaultDicexpStyleProviders(),
  backgroundColor: BACKGROUND_COLOR,
  widgetOwnerClass: WIDGET_OWNER_CLASS,
  Loading: () => "loading…",
  ErrorAlert: () => "error!",
  tagNameForStepsRepresentation: "steps-representation",
});

declare module "solid-js" {
  namespace JSX {
    interface IntrinsicElements {
      "ro-widget-ref-link": { address: string };
      "ro-widget-dicexp": { code: string; evaluation?: DicexpEvaluation };
      "ro-widget-dicexp-no-runtime": {
        code: string;
        evaluation?: DicexpEvaluation;
      };
    }
  }
}

const App: Component = () => (
  <div class={styles.App}>
    <div class="grid grid-cols-2 bg-slate-950">
      <div>
        <Left />
      </div>
      <div>
        <Right />
      </div>
    </div>
  </div>
);
export default App;

const Left: Component = () => {
  let ownerEl!: HTMLDivElement,
    anchorEl!: HTMLDivElement;

  onMount(() => {
    const controller = registerWidgetOwnerEx(anchorEl);
    const observer = new ResizeObserver(() => controller.nofityLayoutChange());
    ownerEl.querySelectorAll(".resize-observee ").forEach((el) =>
      observer.observe(el)
    );
  });

  const forgedResults: DicexpEvaluation[] = [
    { result: ["value", 42], repr: ["vp", 42] },
    {
      result: ["value", 42],
      repr: ["vp", 42],
      statistics: { timeConsumption: { ms: 1 } },
    },
    {
      result: ["value", 42],
      repr: ["vp", 42],
      environment: ["dicexp@0.4.1", `{r:42,s:"0.4.0"}`],
      statistics: { timeConsumption: { ms: 1 } },
    },
    {
      result: ["value", 42],
      repr: ["vp", 42],
      environment: ["dicexp@0.4.1", `{r:42,s:"0.4.0"}`],
    },
    {
      result: ["value", 42],
      repr: ["vp", 42],
      environment: ["dicexp@0.4.1", `{r:42,s:"0.4.0"}`],
      statistics: { timeConsumption: { ms: 1 } },
      location: "local",
    },
    {
      result: ["value", 42],
      repr: ["vp", 42],
      environment: ["dicexp@0.4.1", `{r:42,s:"0.4.0"}`],
      statistics: { timeConsumption: { ms: 1 } },
      location: "server",
    },
    { result: ["value_summary", "四十二"], repr: ["vp", 42] },
    { result: "error", repr: ["e", "error"] },
    { result: ["error", "?"], repr: ["e", "?"] },
    { result: ["error", new Error("?")], repr: ["e", "?"] },
  ];

  return (
    <div ref={ownerEl} class={`${WIDGET_OWNER_CLASS}`}>
      <div ref={anchorEl} class="relative z-10" />
      <div class="flex flex-col min-h-screen">
        <div class="h-[33vh]">
          <ro-widget-dicexp code="d100" />
        </div>
        <div class="h-[33vh] p-4 overflow-y-scroll">
          <div class="h-screen bg-slate-900">
            <LeftInner />
          </div>
        </div>
        <div class="grid grid-cols-2">
          <div>
            <div class="resize-observee flex flex-col">
              <ro-widget-dicexp code="d100" />
              <Index each={forgedResults}>
                {(result) => (
                  <ro-widget-dicexp code="d100" evaluation={result()} />
                )}
              </Index>
            </div>
          </div>
          <div>
            <div class="resize-observee flex flex-col">
              <ro-widget-dicexp-no-runtime code="d100" />
              <Index each={forgedResults}>
                {(result) => (
                  <ro-widget-dicexp-no-runtime
                    code="d100"
                    evaluation={result()}
                  />
                )}
              </Index>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

const LeftInner: Component = () => {
  let anchorEl!: HTMLDivElement;

  onMount(() => registerWidgetOwnerEx(anchorEl));

  return (
    <div
      class={`${WIDGET_OWNER_CLASS} bg-slate-800`}
    >
      <div ref={anchorEl} class="relative z-10" />
      <ro-widget-dicexp code="d100" />
      <ro-widget-dicexp code="d100" />
      <div class="h-[50vh]" />
      <ro-widget-dicexp code="d100" />
      <ro-widget-dicexp code="d100" />
    </div>
  );
};

const Right: Component = () => {
  let anchorEl!: HTMLDivElement;

  const [shouldDisplay, setShouldDisplay] = createSignal(true);

  onMount(() => registerWidgetOwnerEx(anchorEl));

  return (
    <div class={`${WIDGET_OWNER_CLASS} bg-stone-950`}>
      <div ref={anchorEl} class="relative z-10" />
      <div class="h-[50vh]" />
      <label>
        <input
          type="checkbox"
          checked={shouldDisplay()}
          onClick={() => setShouldDisplay(!shouldDisplay())}
        />显示
      </label>
      <div style={{ display: shouldDisplay() ? undefined : "none" }}>
        <ro-widget-ref-link address="TP.foo" />
        <ro-widget-dicexp code="d100" />
        <ro-widget-dicexp code="d100" />
        <ro-widget-dicexp
          code="d100"
          evaluation={{ result: ["value", 42], repr: ["vp", 42] }}
        />
      </div>
    </div>
  );
};

function registerWidgetOwnerEx(anchorEl: HTMLElement) {
  const ownerEl: HTMLElement = anchorEl.closest("." + WIDGET_OWNER_CLASS)!;
  const controller = registerRoWidgetOwner(
    ownerEl,
    { widgetAnchorElement: anchorEl, level: 1 },
  );
  const o = new ResizeObserver(() => controller.nofityLayoutChange());
  o.observe(ownerEl);

  return controller;
}
