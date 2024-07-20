import { bindings } from "./bindings";

const textEncoder = new TextEncoder();

export interface ParseAndRenderResult {
  html: string;
  devEventsInDebugFormat?: string;
}

export function parseAndRender(
  input: string,
): ParseAndRenderResult {
  let result = bindings.parse_and_render(textEncoder.encode(input));
  return {
    html: result.clone_html(),
    ...("clone_dev_events_in_debug_format" in result
      ? {
        devEventsInDebugFormat:
          (result.clone_dev_events_in_debug_format as () => string)(),
      }
      : {}),
  };
}
