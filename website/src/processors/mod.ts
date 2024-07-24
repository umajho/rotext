import { LookupListRaw } from "../pages/Playground/preview-parts/Preview/internal-types";

export interface RotextProcessor {
  parseAndRender(input: string): RotextProcessResult;
}

export interface RotextProcessResult {
  error: Error | null;
  lookupListRaw: LookupListRaw;
  parsingTimeMs?: number;
}
