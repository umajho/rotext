import { bindings } from "./bindings";

const textEncoder = new TextEncoder();

export interface ParseAndRenderOptions {
  tag_name_map: TagNameMap;
  should_include_block_ids: boolean;
}

export interface TagNameMap {
  block_call_error: string;
  code_block: string;
  ref_link: string;
  dicexp: string;
  wiki_link: string;
}

export interface ParseAndRenderResult {
  html: string;
  block_id_to_lines_map: Record<number, [number, number]>;
  dev_events_in_debug_format?: string;
}

export function parseAndRender(
  input: string,
  opts: ParseAndRenderOptions,
): ["ok", ParseAndRenderResult] | ["error", string] {
  for (const name of Object.values(opts.tag_name_map)) {
    if (!isValidTagName(name)) {
      throw new Error(`invalid tag name: ${name}`);
    }
  }

  let output: Uint8Array;
  try {
    output = bindings.parse_and_render(
      textEncoder.encode(input),
      textEncoder.encode(JSON.stringify(opts)),
    );
  } catch (error) {
    return ["error", error as string];
  }

  return ["ok", JSON.parse(new TextDecoder().decode(output))];
}

function isValidTagName(name: string) {
  return /^[0-9a-z-]+$/i.test(name) && !/(^-)|(-$)|--/.test(name);
}
