import fs from "node:fs/promises";
import path from "node:path";

import {
  type HTMLElement,
  type Node,
  parse as parseHTML,
} from "node-html-parser"; // TODO!!: 最终去除此依赖。

import { Navigation } from "../src/types/navigation";

import { parseMarkdown } from "./internal/marked";

const PATHS = (() => {
  const STATIC_FOLDER = path.join(__dirname, "../public/static");

  const SYNTAX_REFERENCE_FOLDER = process.env["SYNTAX_REFERENCE_PATH"];
  if (!SYNTAX_REFERENCE_FOLDER) {
    throw new Error('missing env "SYNTAX_REFERENCE_PATH"');
  }
  const NAVIGATION_FILE = path.join(SYNTAX_REFERENCE_FOLDER, "navigation.md");

  const GENERATED_FOLDER = path.join(STATIC_FOLDER, "generated");
  const GENERATED_SYNTAX_REFERENCE_FOLDER = //
    path.join(GENERATED_FOLDER, "syntax-reference");

  return {
    STATIC_FOLDER,
    SYNTAX_REFERENCE_FOLDER,
    NAVIGATION_FILE,
    GENERATED_FOLDER,
    GENERATED_SYNTAX_REFERENCE_FOLDER,
  } as const;
})();

async function main() {
  const navigation = await (async () => {
    const result = await buildNavigation({
      navigationFilePath: PATHS.NAVIGATION_FILE,
    });
    if (result[0] === "error") {
      console.error(`error building navigation: ${result[1]}`);
      process.exit(1);
    }
    return result[1];
  })();

  const targetsInNavigation = (() => {
    const result = collectTargetsInNavigation(navigation, {
      allowsDuplication: false,
    });
    if (result[0] === "error") {
      console.error(`error collecting occurred names: ${result[1]}`);
      process.exit(1);
    }
    return result[1];
  })();

  const infos = await (async () => {
    const result = await collectFileInfos({
      rootFolderPath: PATHS.SYNTAX_REFERENCE_FOLDER,
      excludedFileNamesNoExtensions: new Set(["navigation"]),

      ignoredHeadings: new Set(["示例"]),

      targetsInNavigation,
    });
    if (result[0] === "error") {
      console.error(`error building indices: ${result[1]}`);
      process.exit(1);
    }
    return result[1];
  })();

  const filesNotLinkedByNavigation = Object.entries(infos)
    .filter(
      ([_, info]) =>
        [...info.headings].every((h) => !targetsInNavigation.has(h)),
    ).map(([filePath, _]) => filePath);
  if (filesNotLinkedByNavigation.length) {
    console.error(
      `files not linked by navigation: ${
        filesNotLinkedByNavigation.map((f) => `“${f}”`).join(" ")
      }`,
    );
    process.exit(1);
  }

  await fs.rm(PATHS.GENERATED_SYNTAX_REFERENCE_FOLDER, {
    recursive: true,
    force: true,
  });
  await fs.mkdir(PATHS.GENERATED_SYNTAX_REFERENCE_FOLDER, { recursive: true });

  await writeFiles({ navigation, infos });
}

async function buildNavigation(
  opts: { navigationFilePath: string },
): Promise<["ok", Navigation] | ["error", string]> {
  const navigationText = (await fs.readFile(opts.navigationFilePath))
    .toString();
  return parseNavigationHTML(navigationText);
}

function parseNavigationHTML(
  navigationText: string,
): ["ok", Navigation] | ["error", string] {
  const navigationHTML = parseHTML(parseMarkdown(navigationText));

  const headingsResult = extractHeadings(navigationHTML, {
    allowsOtherElements: false,
    ensuresHierarchy: true,
    allowedContent: {
      singleTextNode: true,
      singleInternalLink: true,
    },
  });
  if (headingsResult[0] === "error") return headingsResult;
  const headings = headingsResult[1];

  const stack: Navigation[] = [];

  for (const heading of headings) {
    while (stack.length > heading.level - 1) {
      stack.pop();
    }

    const name = heading.child.innerText;
    const realName = (heading.child as HTMLElement).getAttribute?.("page-name");

    const nav: Navigation = {
      ...(heading.child.rawTagName ? {} : { isPlain: true }),
      name,
      ...(realName ? { realName } : {}),
    };
    if (stack.length > 0) {
      const top = stack.at(-1)!;
      top.children ??= [];
      if (!top.children.length) {
        if (!top.isPlain) {
          return ["error", `branch navigation “${top.name}” is not plain`];
        }
      }
      top.children.push(nav);
    }
    stack.push(nav);
  }

  return ["ok", stack[0]!];
}

interface Heading {
  level: number;
  child: Node;
}

function extractHeadings(dom: HTMLElement, opts: {
  allowsOtherElements: boolean;
  ensuresHierarchy: true;
  allowedContent: {
    singleTextNode: true;
    singleInternalLink: boolean;
  };
}): ["ok", Heading[]] | ["error", string] {
  let currentLevel = 0;
  const headings: Heading[] = [];

  for (const node of dom.childNodes.filter((node) => !isBlankTextNode(node))) {
    const headingLevelMatch = /^h([1-6])$/.exec(node.rawTagName);
    if (!headingLevelMatch) {
      if (opts.allowsOtherElements) continue;
      return [
        "error",
        (node.rawTagName ? `\`${node.rawTagName}\`` : "text node") +
        " is not supported on top level",
      ];
    }
    const headingLevel = Number(headingLevelMatch[1]);

    if (opts.ensuresHierarchy) {
      if (headingLevel === 1 && currentLevel !== 0) {
        return ["error", "there should only be one `h1`"];
      }
      if (headingLevel > currentLevel + 1) {
        return [
          "error",
          `heading level jumped from ${currentLevel} to ${headingLevel}`,
        ];
      }
    }
    currentLevel = headingLevel;

    const childNodes = node.childNodes.filter((node) => !isBlankTextNode(node));
    if (childNodes.length !== 1) {
      return ["error", "a heading should contain exactly one child"];
    }

    const child = childNodes[0]!;
    let isChildValid = false;
    if (opts.allowedContent.singleTextNode && !child.rawTagName) {
      isChildValid = true;
    } else if (
      opts.allowedContent.singleInternalLink &&
      child.rawTagName === "x-internal-link"
    ) {
      isChildValid = true;
    }
    if (!isChildValid) {
      return [
        "error",
        `not a valid child for heading: \`${child.toString()}\``,
      ];
    }

    headings.push({ level: headingLevel, child });
  }

  return ["ok", headings];
}

function isBlankTextNode(node: Node): boolean {
  return !node.rawTagName && !node.rawText.trim();
}

function collectTargetsInNavigation(nav: Navigation, opts: {
  allowsDuplication: false;
}): ["ok", Set<string>] | ["error", string] {
  const set = new Set<string>();

  if (!nav.isPlain) {
    set.add(nav.realName ?? nav.name);
  }

  for (const child of nav.children ?? []) {
    const childResult = collectTargetsInNavigation(child, opts);
    if (childResult[0] === "error") return childResult;
    for (const name of childResult[1]) {
      if (set.has(name)) return ["error", `duplicate name: ${name}`];
      set.add(name);
    }
  }

  return ["ok", set];
}

interface CollectFileInfosOptions {
  rootFolderPath: string;
  excludedFileNamesNoExtensions: Set<string>;

  ignoredHeadings: Set<string>;

  targetsInNavigation: Set<string>;
}

interface FileInfo {
  headings: Set<string>;
  internalLinkTargets: Set<string>;

  html: HTMLElement;
}

async function collectFileInfos(
  opts: CollectFileInfosOptions,
): Promise<["ok", { [name: string]: FileInfo }] | ["error", string]> {
  const filePaths = (await fs.readdir(opts.rootFolderPath, { recursive: true }))
    .filter((p) =>
      path.extname(p) === ".md" &&
      !opts.excludedFileNamesNoExtensions.has(path.parse(p).name)
    );

  const infos: { [name: string]: FileInfo } = {};

  const seenHeadings = new Set<string>();

  for (const filePath of filePaths) {
    const fileText =
      (await fs.readFile(path.join(opts.rootFolderPath, filePath))).toString();
    const fileHTML = parseHTML(parseMarkdown(fileText));

    const headingsResult = extractHeadings(fileHTML, {
      allowsOtherElements: true,
      ensuresHierarchy: true,
      allowedContent: {
        singleTextNode: true,
        singleInternalLink: false,
      },
    });
    if (headingsResult[0] === "error") {
      return ["error", `error collecting headings: ${headingsResult[1]}`];
    }
    const headings = headingsResult[1];
    const indexableHeadings = headings
      .map((h) => h.child.rawText.trim())
      .filter((h) => !opts.ignoredHeadings.has(h));
    if (indexableHeadings[0] !== path.parse(filePath).name) {
      return ["error", `file name not matching with \`h1\` content`];
    }

    for (const heading of indexableHeadings) {
      if (seenHeadings.has(heading)) {
        return ["error", `duplicate heading: “${heading}”`];
      }
      seenHeadings.add(heading);
    }

    const internalLinkTargets = new Set<string>();
    for (const el of fileHTML.querySelectorAll("x-internal-link")) {
      const target = el.getAttribute("page-name") ?? el.textContent;
      internalLinkTargets.add(target);
    }

    infos[filePath] = {
      headings: new Set(indexableHeadings),
      internalLinkTargets,

      html: fileHTML,
    };
  }

  const missingTargets = new Set<string>();
  for (const target of [...opts.targetsInNavigation]) {
    if (!seenHeadings.has(target)) {
      missingTargets.add(target);
    }
  }
  for (const [_, { internalLinkTargets }] of Object.entries(infos)) {
    for (const target of internalLinkTargets) {
      if (!seenHeadings.has(target)) {
        missingTargets.add(target);
      }
    }
  }
  if (missingTargets.size) {
    return [
      "error",
      `missing internal link targets: ${[...missingTargets].join(", ")}`,
    ];
  }

  return ["ok", infos];
}

async function writeFiles(
  opts: { navigation: Navigation; infos: { [name: string]: FileInfo } },
) {
  await fs.writeFile(
    path.join(PATHS.GENERATED_SYNTAX_REFERENCE_FOLDER, "navigation.json"),
    JSON.stringify(opts.navigation),
  );

  await fs.writeFile(
    path.join(PATHS.GENERATED_SYNTAX_REFERENCE_FOLDER, "files-headings.json"),
    JSON.stringify(
      Object.fromEntries(
        Object.entries(opts.infos).map((
          [k, v],
        ) => [removeFilePathExtension(k), [...v.headings]]),
      ),
    ),
  );

  for (const [filePath, info] of Object.entries(opts.infos)) {
    const renderedPath = path.join(
      PATHS.GENERATED_SYNTAX_REFERENCE_FOLDER,
      getRenderedFilePath(filePath),
    );
    await fs.mkdir(path.dirname(renderedPath), { recursive: true });
    await fs.writeFile(renderedPath, info.html.outerHTML);
  }
}

function getRenderedFilePath(originalPath: string) {
  const p = path.parse(originalPath);
  return path.format({
    dir: p.dir,
    name: p.name,
    ext: ".inc.html",
  });
}

function removeFilePathExtension(originalPath: string) {
  const p = path.parse(originalPath);
  return path.format({
    dir: p.dir,
    name: p.name,
  });
}

await main();
