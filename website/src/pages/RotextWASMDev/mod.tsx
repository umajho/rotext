import { Component, createEffect, createSignal, on } from "solid-js";

import pretty from "pretty";

import { makeOnlyInstance as makeOnlyRotextAdapterInstance } from "@rotext/wasm-bindings-adapter";
import initRotextWASM, * as rotextBindings from "rotext_wasm_bindings";

import { Card } from "../../components/ui/mod";

const rotextAdapter = await (async () => {
  await initRotextWASM();
  return makeOnlyRotextAdapterInstance(rotextBindings);
})();

export default (() => {
  const [input, setInput] = createSignal("Hello, world!");
  const [result, setResult] = createSignal("Loading…");

  createEffect(
    on([input], ([input]) => {
      console.time("rotext RS (dev)");
      const result = rotextAdapter.dev(input);
      console.timeEnd("rotext RS (dev)");
      setResult(pretty(result));
    }),
  );

  return (
    <Card>
      <textarea
        style="field-sizing: content;"
        value={input()}
        onInput={(ev) => setInput(ev.currentTarget.value)}
      >
      </textarea>
      <pre class="whitespace-pre-wrap break-all">{result()}</pre>
    </Card>
  );
}) satisfies Component;
