import fs from "node:fs/promises";
import path from "node:path";

const FOLDER_PATH = path.join(__dirname, "../../../docs/wiki/语法参考");

async function main() {
  const pageNames = (await fs.readdir(FOLDER_PATH, { recursive: true }))
    .filter((p) => path.extname(p) === ".md");
  for (const name of pageNames) {
    const p = path.join(FOLDER_PATH, name);
    console.log(name);
    await replaceInFile(p);
  }
}

async function replaceInFile(p: string) {
  let content = await fs.readFile(p, "utf8");
  content = content.replace(/\[\[(.*?)\]\]/g, (_, ...[addr]: [string]) => {
    if (!addr.startsWith("#")) {
      return `[[s:${addr}]]`;
    }
    return `[[${addr}]]`;
  });
  await fs.writeFile(p, content);
}

main();
