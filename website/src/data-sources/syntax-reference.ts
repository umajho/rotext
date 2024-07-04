import { createMemo, createResource } from "solid-js";

export const [syntaxReferenceFilesToHeadingMap] = createResource(async () => {
  const path = "static/generated/syntax-reference/files-headings.json";
  return (await (await fetch(path)).json()) as //
  { [filePath: string]: string[] };
});

export const syntaxReferenceIndex = createMemo(() => {
  const map = syntaxReferenceFilesToHeadingMap();
  if (!map) return;

  const index: { [heading: string]: string } = {};
  for (const [filePath, headings] of Object.entries(map)) {
    for (const heading of headings) {
      index[heading] = filePath;
    }
  }

  return index;
});
