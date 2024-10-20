import { format } from "hast-util-format";
import { fromHtml } from "hast-util-from-html";
import { toHtml } from "hast-util-to-html";

export function formatHTML(
  input: string,
): string {
  const tree = fromHtml(input, { fragment: true });
  format(tree);
  return toHtml(tree).trim();
}
