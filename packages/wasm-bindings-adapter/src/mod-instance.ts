import { bindings } from "./bindings";

const textEncoder = new TextEncoder();

type BlockIDAndLinesPair = [id: string, range: { start: number; end: number }];

export interface ParseAndRenderResult {
  html: string;
  blockIDAndLinesPairs: BlockIDAndLinesPair[];
  devEventsInDebugFormat?: string;
}

export function parseAndRender(
  input: string,
): ParseAndRenderResult {
  const result = bindings.parse_and_render(textEncoder.encode(input));

  const blockIDToLinesMap = deserializeBlockIDToLinesMap(
    result.clone_block_id_to_lines_map(),
  );

  return {
    html: result.clone_html(),
    blockIDAndLinesPairs: blockIDToLinesMap,
    ...("clone_dev_events_in_debug_format" in result
      ? {
        devEventsInDebugFormat:
          (result.clone_dev_events_in_debug_format as () => string)(),
      }
      : {}),
  };
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
