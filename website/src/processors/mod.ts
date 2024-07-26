export interface RotextProcessor {
  process(input: string): RotextProcessResult;
}

export interface RotextProcessResult {
  html: string | null;
  error: Error | null;

  parsingTimeMs?: number;
}
