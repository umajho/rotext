import { bindings } from "./bindings";

export interface ParseAndRenderInput {
  input: string;
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
  block_id_to_lines_map: Map<number, [number, number]>;
  dev_events_in_debug_format?: string;
}

export function parseAndRender(
  input: ParseAndRenderInput,
): ["ok", ParseAndRenderResult] | ["error", string] {
  for (const name of Object.values(input.tag_name_map)) {
    if (!isValidTagName(name)) {
      throw new Error(`invalid tag name: ${name}`);
    }
  }

  let output: any;
  try {
    output = bindings.parse_and_render(
      input,
    );
  } catch (error) {
    return ["error", error as string];
  }

  return ["ok", output];
}

function isValidTagName(name: string) {
  return /^[0-9a-z-]+$/i.test(name) && !/(^-)|(-$)|--/.test(name);
}
