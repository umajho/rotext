import { bindings } from "./bindings";

const textEncoder = new TextEncoder();

type BlockIDAndLinesPair = [id: string, range: { start: number; end: number }];

export interface ParseAndRenderResult {
  html: string;
  blockIDAndLinesPairs: BlockIDAndLinesPair[];
  devEventsInDebugFormat?: string;
}

export interface ParseAndRenderOptions {
  tagNameMap: TagNameMap;
  shouldIncludeBlockIDs: boolean;
}

export interface TagNameMap {
  "code-block": string;
  "ref-link": string;
  "dicexp": string;
  "internal-link": string;
}

export function parseAndRender(
  input: string,
  opts: ParseAndRenderOptions,
): ["ok", ParseAndRenderResult] | ["error", string] {
  const result = bindings.parse_and_render(
    textEncoder.encode(input),
    serializeTagNameMap(opts.tagNameMap),
    opts.shouldIncludeBlockIDs,
  );

  const error = result.clone_error();
  if (error) {
    return ["error", error];
  }

  const output = result.clone_ok()!;

  const blockIDToLinesMap = deserializeBlockIDToLinesMap(
    output.clone_block_id_to_lines_map(),
  );

  const ret = {
    html: output.clone_html(),
    blockIDAndLinesPairs: blockIDToLinesMap,
    ...("clone_dev_events_in_debug_format" in output
      ? {
        devEventsInDebugFormat:
          (output.clone_dev_events_in_debug_format as () => string)(),
      }
      : {}),
  };
  return ["ok", ret];
}

function serializeTagNameMap(tagNameMap: TagNameMap): string {
  return [
    tagNameMap["code-block"],
    tagNameMap["ref-link"],
    tagNameMap["dicexp"],
    tagNameMap["internal-link"],
  ].join("\0");
}

function deserializeBlockIDToLinesMap(input: string): BlockIDAndLinesPair[] {
  if (!input) return [];

  return input
    .split(";")
    .map((x): BlockIDAndLinesPair => {
      const idAndRange = x.split(":");
      const id = idAndRange[0]!;
      const range = idAndRange[1]?.split("-")!;
      return [id, {
        start: Number(range[0]),
        end: Number(range[1]),
      }];
    });
}
