import type { Navigator } from "@solidjs/router";

const stuff: {
  navigator?: Navigator;
} = {};

export default stuff;

export function initializeGlobal(initialStuff: Required<typeof stuff>) {
  Object.assign(stuff, initialStuff);
}
