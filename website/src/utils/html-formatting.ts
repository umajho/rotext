export function formatHTML(
  input: string,
  opts: { formatter: (input: string) => string },
) {
  return opts.formatter(input)
    // workarounds
    .replace(/<br>/g, "<br />")
    .replace(/<hr>/g, "<hr />");
}
