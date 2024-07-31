# @rotext/wasm-bindings-adapter

WIP

```typescript
import { makeOnlyInstance } from "@rotext/wasm-bindings-adapter";
import init, * as bindings from "rotext_wasm_bindings";

await init();
const rotextAdapter = makeOnlyInstance(bindings);

// …
```
