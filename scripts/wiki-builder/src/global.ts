import fs from "node:fs/promises";
import path from "node:path";

import {
  makeOnlyInstance as makeOnlyRotextAdapterInstance,
} from "@rotext/wasm-bindings-adapter";
import initRotextWASM, * as rotextBindings from "rotext_wasm_bindings";

// 不知道是 node 还是 vite-node 的问题，直接调用 `initRotextWASM` 会导致 undici
// 报错：“TypeError: fetch failed”。甚至用 deno 执行都完全没有问题（而且还很快）。
// 可惜考虑到让 CI 同时包含两个 JS 运行时也太臃肿了，还是继续只用 node 好了。等
// 哪一天 deno 发展到能支持这个项目的各种情况时，就全盘换成 deno 吧。
const wasmBuffer = await fs.readFile(
  path.join(
    // @ts-ignore
    import.meta.dirname,
    "../node_modules/rotext_wasm_bindings/rotext_wasm_bindings_bg.wasm",
  ),
);

export const rotextAdapter = await (async () => {
  await initRotextWASM(wasmBuffer);
  return makeOnlyRotextAdapterInstance(rotextBindings);
})();
