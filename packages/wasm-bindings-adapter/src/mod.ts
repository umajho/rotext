import type * as RotextWASMBindingsModule from "rotext_wasm_bindings";

import { bindings, setBindings } from "./bindings";
import * as modInstance from "./mod-instance";

export type {
  Extension,
  ParseAndRenderResult,
  TagNameMap,
} from "./mod-instance";

export async function makeOnlyInstance(
  bindingsIn: typeof RotextWASMBindingsModule,
) {
  if (bindings) {
    throw new Error("There can only be a single Rotext WASM bridge instance.");
  }

  setBindings(bindingsIn);

  return modInstance;
}
