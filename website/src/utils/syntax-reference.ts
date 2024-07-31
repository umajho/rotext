export function getSyntaxReferencePathOfHeading(
  heading: string,
  opts: { index: { [heading: string]: string } },
): { path: string; pathWithAnchor: string } {
  const fileName = opts.index[heading]!;
  const path = `/syntax-reference/${fileName}`;
  let pathWithAnchor = path;
  if (!fileName.endsWith(`/${heading}`)) {
    pathWithAnchor += `#${heading}`;
  }

  return { path, pathWithAnchor };
}
