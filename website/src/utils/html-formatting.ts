export function formatHTML(
  input: string,
  opts: { formatter: (input: string) => string },
) {
  return opts.formatter(input);
}
