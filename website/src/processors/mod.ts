import { LookupListRaw } from "../pages/Playground/preview-parts/Preview/internal-types";

export interface RotextProcessorProcessOptions {
  requiresLookupListRaw: boolean;
  tagNameMap: TagNameMap;
}

export interface TagNameMap {
  "code-block": string;
  "ref-link": string;
  "dicexp": string;
  "internal-link": string;
}

export interface RotextProcessor {
  process(
    input: string,
    opts: RotextProcessorProcessOptions,
  ): RotextProcessResult;
}

export interface RotextProcessResult {
  html: string | null;
  error: Error | null;

  parsingTimeMs: number | null;

  /**
   * 额外信息，会在实验场预览部分以附加标签的形式呈现。
   */
  extraInfos: { name: string; content: string }[];

  lookupListRawCollector: ((outputEl: HTMLElement) => LookupListRaw) | null;
}
