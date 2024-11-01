import fs from "node:fs/promises";
import path from "node:path";

import titleToHeadingsMap from "../../public/static/generated/syntax-reference/files-headings.json" with {
  type: "json",
};

const headingToTitleMap = (() => {
  const m: Record<string, string> = {};
  for (const [title, headings] of Object.entries(titleToHeadingsMap)) {
    for (const heading of headings) {
      m[heading] = title;
    }
  }
  return m;
})();

const FOLDER_PATH = path.join(__dirname, "../../../docs/语法参考");

async function main() {
  const filePaths = (await fs.readdir(FOLDER_PATH, { recursive: true }))
    .filter((p) =>
      path.extname(p) === ".md" && path.basename(p) !== "navigation.md"
    )
    .map((p) => path.join(FOLDER_PATH, p));
  for (const p of filePaths) {
    console.log(path.basename(p));
    await replaceInFile(p);
  }
}

async function replaceInFile(p: string) {
  let content = await fs.readFile(p, "utf8");
  content = content.replace(/\[\[(.*?)\]\]/g, (_, ...[addr]: [string]) => {
    let [actual, shown] = addr.split("|", 2) as [string, string | undefined];
    if (!(actual in headingToTitleMap)) {
      console.warn(`${JSON.stringify(actual)} not in map`);
      return `[[${addr}]]`;
    }
    let newActual = headingToTitleMap[actual]!;
    if (actual !== newActual) {
      if (newActual.split("/").at(-1) !== actual) {
        newActual += `#${actual}`;
      }
      if (!shown) {
        shown = actual;
      }
      // console.info(`${actual} -> ${newActual}`);
    }
    if (shown) {
      return `[[${newActual}|${shown}]]`;
    } else {
      return `[[${newActual}]]`;
    }
  });
  await fs.writeFile(p, content);
}

await main();
