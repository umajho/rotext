import type * as RotextWASMBindingsModule from "rotext_wasm_bindings";

export let bindings!: typeof RotextWASMBindingsModule;

export function setBindings(bindingsIn: typeof RotextWASMBindingsModule) {
  bindings = bindingsIn;
}
