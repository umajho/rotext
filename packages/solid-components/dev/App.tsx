import "./App.css";
import styles from "./App.module.css";

import { type Component, Index, onMount } from "solid-js";

import { registerCustomElementForStepsRepresentation } from "@dicexp/solid-components";

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
  withStyle: withDefaultRefLinkStyle,
  backgroundColor: BACKGROUND_COLOR,
  widgetOwnerClass: WIDGET_OWNER_CLASS,
  refContentRenderer: (el, addr, onAddressChange) => {
    el.innerText = JSON.stringify(addr);
    onAddressChange(() => el.innerText = JSON.stringify(addr));
  },
});
registerCustomElementForStepsRepresentation("steps-representation");
registerCustomElementForRoWidgetDicexp("ro-widget-dicexp", {
  withStyle: withDefaultDicexpStyle,
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
  withStyle: withDefaultDicexpStyle,
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
      <Left />
      <Right />
    </div>
  </div>
);
export default App;

const Left: Component = () => {
  let anchorEl!: HTMLDivElement;

  onMount(() => registerWidgetOwner(anchorEl));

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
    <div class={`${WIDGET_OWNER_CLASS}`}>
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
          <div class="flex flex-col">
            <ro-widget-dicexp code="d100" />
            <Index each={forgedResults}>
              {(result) => (
                <ro-widget-dicexp code="d100" evaluation={result()} />
              )}
            </Index>
          </div>
          <div class="flex flex-col">
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
  );
};

const LeftInner: Component = () => {
  let anchorEl!: HTMLDivElement;

  onMount(() => registerWidgetOwner(anchorEl));

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

  onMount(() => registerWidgetOwner(anchorEl));

  return (
    <div class={`${WIDGET_OWNER_CLASS} bg-stone-950`}>
      <div ref={anchorEl} class="relative z-10" />
      <div class="h-[50vh]" />
      <ro-widget-ref-link address="TP.foo" />
      <ro-widget-dicexp code="d100" />
      <ro-widget-dicexp code="d100" />
    </div>
  );
};

function registerWidgetOwner(anchorEl: HTMLElement) {
  const ownerEl: HTMLElement = anchorEl.closest("." + WIDGET_OWNER_CLASS)!;
  const controller = registerRoWidgetOwner(
    ownerEl,
    { widgetAnchorElement: anchorEl, level: 1 },
  );
  const o = new ResizeObserver(() => controller.nofityLayoutChange());
  o.observe(ownerEl);
}
