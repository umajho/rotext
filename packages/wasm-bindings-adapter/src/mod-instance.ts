import { bindings } from "./bindings";

const textEncoder = new TextEncoder();

export interface ParseAndRenderResult {
  html: string;
  block_id_to_lines_map: Map<number, [number, number]>;
  dev_events_in_debug_format?: string;
}

export interface ParseAndRenderOptions {
  tagNameMap: TagNameMap;
  shouldIncludeBlockIDs: boolean;
}

export interface TagNameMap {
  "block-call-error": string;
  "code-block": string;
  "ref-link": string;
  "dicexp": string;
  "wiki-link": string;
}

export function parseAndRender(
  input: string,
  opts: ParseAndRenderOptions,
): ["ok", ParseAndRenderResult] | ["error", string] {
  let output: any;
  try {
    output = bindings.parse_and_render(
      textEncoder.encode(input),
      serializeTagNameMap(opts.tagNameMap),
      opts.shouldIncludeBlockIDs,
    );
  } catch (error) {
    return ["error", error as string];
  }

  return ["ok", output];
}

function serializeTagNameMap(tagNameMap: TagNameMap): string {
  for (const name of Object.values(tagNameMap)) {
    if (!isValidTagName(name)) {
      throw new Error(`invalid tag name: ${name}`);
    }
  }

  return [
    tagNameMap["block-call-error"],
    tagNameMap["code-block"],
    tagNameMap["ref-link"],
    tagNameMap["dicexp"],
    tagNameMap["wiki-link"],
  ].join("\0");
}

function isValidTagName(name: string) {
  return /^[0-9a-z-]+$/i.test(name) && !/(^-)|(-$)|--/.test(name);
}
