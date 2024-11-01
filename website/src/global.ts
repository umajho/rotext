import type { Navigator } from "@solidjs/router";

const stuff: {
  currentPageName?: string | null;
  navigator?: Navigator;
} = {};

export default stuff;

export function initializeGlobal(initialStuff: Required<typeof stuff>) {
  Object.assign(stuff, initialStuff);
}

export function updateGlobalCurrentPageName(name: string | null) {
  stuff.currentPageName = name;
}
