import fs from "node:fs/promises";
import path from "node:path";

const FOLDER_PATH = path.join(__dirname, "../../docs/语法参考");

async function main() {
  const pageNames = (await fs.readdir(FOLDER_PATH, { recursive: true }))
    .filter((p) =>
      path.extname(p) === ".md" && path.basename(p) !== "navigation.md"
    );
  for (const name of pageNames) {
    const p = path.join(FOLDER_PATH, name);
    console.log(name);
    await replaceInFile(p, /(.*)\.md$/.exec(name)![1]!);
  }
}

async function replaceInFile(p: string, name: string) {
  let content = await fs.readFile(p, "utf8");
  content = content.replace(/\[\[(.*?)\]\]/g, (_, ...[addr]: [string]) => {
    let [actual, shown] = addr.split("|", 2) as [string, string | undefined];
    if (actual === `${name}#${shown}`) {
      console.log("x", actual);
      return `[[#${shown}|${shown}]]`;
    }
    return `[[${addr}]]`;
  });
  await fs.writeFile(p, content);
}

main();
