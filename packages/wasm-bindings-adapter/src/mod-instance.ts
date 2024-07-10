import { bindings } from "./bindings";

const textEncoder = new TextEncoder();

export function parse(input: string): number {
  return bindings.parse(textEncoder.encode(input));
}
