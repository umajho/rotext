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

import pretty from "pretty";

import {
  makeOnlyInstance as makeOnlyRotextAdapterInstance,
  ParseAndRenderResult,
} from "@rotext/wasm-bindings-adapter";
import initRotextWASM, * as rotextBindings from "rotext_wasm_bindings";

import { Card } from "../../components/ui/mod";

const rotextAdapter = await (async () => {
  await initRotextWASM();
  return makeOnlyRotextAdapterInstance(rotextBindings);
})();

type Tab = "html-source" | "events";

export default (() => {
  const [input, setInput] = createSignal("Hello, world!");
  const [result, setResult] = createSignal<ParseAndRenderResult | null>(null);

  const [currentTab, setCurrentTab] = //
    createSignal<Tab>("html-source");

  createEffect(
    on([input], ([input]) => {
      console.time("rotext RS (dev)");
      const result = rotextAdapter.parseAndRender(input);
      console.timeEnd("rotext RS (dev)");
      setResult(result);
    }),
  );

  const prettyResultHTML = createMemo(() => {
    let html = result()?.html;
    if (html === undefined) return undefined;
    return pretty(html);
  });

  const hasDevEventsDebugFormat = () =>
    result()?.devEventsInDebugFormat !== undefined;
  createEffect(on([hasDevEventsDebugFormat], ([hasDevEventsDebugFormat]) => {
    if (!hasDevEventsDebugFormat) {
      setCurrentTab("html-source");
    }
  }));

  return (
    <Card>
      <textarea
        style="field-sizing: content;"
        value={input()}
        onInput={(ev) => setInput(ev.currentTarget.value)}
      >
      </textarea>
      <Show when={result()} fallback={<>"Loading…"</>}>
        <>
          <select
            class="w-fit"
            value={"html-source"}
            onChange={(ev) => setCurrentTab(ev.currentTarget.value as Tab)}
          >
            <option value={"html-source"}>HTML 源代码</option>
            <Show when={hasDevEventsDebugFormat()}>
              <option value={"events"}>各事件</option>
            </Show>
          </select>
          <Switch>
            <Match when={currentTab() === "html-source"}>
              <pre class="whitespace-pre-wrap break-all">{prettyResultHTML()}</pre>
            </Match>
            <Match when={currentTab() === "events"}>
              <pre class="whitespace-pre-wrap break-all">{result()!.devEventsInDebugFormat}</pre>
            </Match>
          </Switch>
        </>
      </Show>
    </Card>
  );
}) satisfies Component;
