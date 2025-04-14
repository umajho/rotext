import "./App.css";
import styles from "./App.module.css";

import { type Component, createSignal, Index } from "solid-js";

import * as Ankor from "ankor";

import { DicexpEvaluation } from "@rotext/solid-components";

import { registerCustomElementForRefLink } from "./ref-link-thingy";
import { registerCustomElementsForDicexp } from "./dicexp-thingy";

registerCustomElementForRefLink();
registerCustomElementsForDicexp();

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
    { result: "error", repr: ["e", "error", undefined] },
    { result: ["error", "runtime", "?"], repr: ["e", "?", undefined] },
    { result: ["error", "parse", "?"] },
  ];

  const widgetOwnerData = JSON.stringify(
    { level: 1 } satisfies Ankor.WidgetOwnerRaw,
  );

  return (
    <div
      class={`${Ankor.WIDGET_OWNER_CLASS}`}
      data-ankor-widget-owner={widgetOwnerData}
    >
      <div class={`${Ankor.ANCHOR_CLASS} relative z-10`} />
      <div class={`${Ankor.CONTENT_CLASS} flex flex-col min-h-screen`}>
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
  const widgetOwnerData = JSON.stringify(
    { level: 1 } satisfies Ankor.WidgetOwnerRaw,
  );

  return (
    <div
      class={Ankor.WIDGET_OWNER_CLASS}
      data-ankor-widget-owner={widgetOwnerData}
    >
      <div
        class={`${Ankor.CONTENT_CLASS} bg-slate-800`}
      >
        <div class={`${Ankor.ANCHOR_CLASS} relative z-10`} />
        <ro-widget-dicexp code="d100" />
        <ro-widget-dicexp code="d100" />
        <div class="h-[50vh]" />
        <ro-widget-dicexp code="d100" />
        <ro-widget-dicexp code="d100" />
      </div>
    </div>
  );
};

const Right: Component = () => {
  const [shouldDisplay, setShouldDisplay] = createSignal(true);

  const widgetOwnerData = JSON.stringify(
    { level: 1 } satisfies Ankor.WidgetOwnerRaw,
  );

  return (
    <div
      class={`${Ankor.WIDGET_OWNER_CLASS} bg-stone-950`}
      data-ankor-widget-owner={widgetOwnerData}
    >
      <div class={`${Ankor.ANCHOR_CLASS} relative z-10`} />
      <div class="h-[50vh]" />
      <label>
        <input
          type="checkbox"
          checked={shouldDisplay()}
          onClick={() => setShouldDisplay(!shouldDisplay())}
        />显示
      </label>
      <div
        class={Ankor.CONTENT_CLASS}
        style={{ display: shouldDisplay() ? undefined : "none" }}
      >
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
