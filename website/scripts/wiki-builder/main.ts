import fs from "node:fs/promises";
import path from "node:path";
import process from "node:process";
import { parseArgs } from "node:util";

import { Index } from "../../src/resource-managers/wiki";

import { discoverNamespaces } from "./discovery";
import { collectNamespace, NamespaceMap } from "./namespaces";
import {
  collectAuthenticFullPageNameToHeadingsMap,
  extractNamespaceAliasMap,
  extractTargetsInNavigation,
  FullPageNameToHeadingsMap,
  getAuthenticFullPageName,
  NamespaceAliasMap,
} from "./utils";

async function main(args_: string[]) {
  const { values: args } = parseArgs({
    options: {
      "input": { type: "string", short: "i" },
      "output": { type: "string", short: "o" },
      "dry-run": { type: "boolean", default: false },
    },
    args: args_,
  });

  if (!args.input || !args.output) throw new Error("bad arguments");

  let hasErrors = false;

  const nsMap: NamespaceMap = new Map() as NamespaceMap;

  for (const ns of await discoverNamespaces(args.input)) {
    const result = await collectNamespace(ns);
    if (result[0] !== "ok") {
      console.error(result[1]);
      hasErrors = true;
      continue;
    }
    nsMap.set(ns.namespace, result[1]);
  }

  if (hasErrors) process.exit(1);

  const nsAliasMap = extractNamespaceAliasMap(nsMap);
  const pageMap = collectAuthenticFullPageNameToHeadingsMap(nsMap);

  const checkResult = checkPages(nsMap, nsAliasMap, pageMap);
  if (checkResult[0] !== "ok") {
    console.error(checkResult[1]);
    process.exit(1);
  }

  if (!args["dry-run"]) {
    await writeFiles({ output: args.output, namespaceMap: nsMap });
  }
}

function checkPages(
  nsMap: NamespaceMap,
  nsAliasMap: NamespaceAliasMap,
  pageMap: FullPageNameToHeadingsMap,
): ["ok"] | ["error", string] {
  const orphans = new Set(Object.keys(pageMap));
  const badTargets: { page?: string; target: string }[] = [];

  for (const [nsName, ns] of nsMap) {
    const targetsInNs = extractTargetsInNavigation(ns.navigation, nsAliasMap);
    for (const target of targetsInNs) {
      const result = checkTarget(target, {
        pageMap,
        namespaceAliasMap: nsAliasMap,

        orphans,
        badTargets,
      });
      if (result[0] !== "ok") return result;
    }

    for (const [pageName, page] of Object.entries(ns.pages)) {
      const currentPageFullName = `${nsName}:${pageName}`;
      for (const target of page.internalLinkTargets) {
        const result = checkTarget(target, {
          pageMap,
          namespaceAliasMap: nsAliasMap,

          orphans,
          badTargets,

          currentPageFullName,
        });
        if (result[0] !== "ok") return result;
      }
    }
  }

  if (orphans.size) {
    const list = [...orphans].map((p) => JSON.stringify(p)).join(", ");
    return ["error", `found orphan pages: ${list}`];
  }

  if (badTargets.length) {
    let list: string[] = [];
    for (const { page, target } of badTargets) {
      let text = JSON.stringify(target);
      if (page !== undefined) {
        text += ` (in ${JSON.stringify(page)})`;
      }
      list.push(text);
    }
    return ["error", `found bad targets: ${list.join(", ")}`];
  }

  return ["ok"];
}

function checkTarget(target: string, opts: {
  namespaceAliasMap: NamespaceAliasMap;
  pageMap: FullPageNameToHeadingsMap;

  orphans: Set<string>;
  badTargets: { page?: string; target: string }[];

  currentPageFullName?: string;
}): ["ok"] | ["error", string] {
  let authenticTarget: string;
  if (target.startsWith("#")) {
    if (!opts.currentPageFullName) throw new Error("unreachable");
    authenticTarget = `${opts.currentPageFullName}${target}`;
  } else {
    const result = getAuthenticFullPageName(target, opts.namespaceAliasMap);
    if (result[0] !== "ok") return result;
    authenticTarget = result[1];
  }

  const [authenticPageName_, heading] = authenticTarget.split("#", 2);
  const authenticPageName = authenticPageName_!;
  if (
    (!opts.pageMap.has(authenticPageName)) ||
    (heading !== undefined &&
      !opts.pageMap.get(authenticPageName)!.has(heading))
  ) {
    opts.badTargets.push({
      page: opts.currentPageFullName,
      target: authenticTarget,
    });
  }
  opts.orphans.delete(authenticPageName);

  return ["ok"];
}

async function writeFiles(opts: {
  output: string;

  namespaceMap: NamespaceMap;
}) {
  for (const [nsName, ns] of opts.namespaceMap) {
    for (const [pageName, page] of Object.entries(ns.pages)) {
      const p = path.join(opts.output, nsName, pageName + ".inc.html");
      await fs.mkdir(path.dirname(p), { recursive: true });
      await fs.writeFile(p, page.html);
    }
  }

  const index: Index = {
    namespaceAliases: Object.fromEntries(
      [...opts.namespaceMap].map(([nsName, ns]) => {
        const aliases = ns.configuration.namespace.aliases;
        if (!aliases) {
          // 既然只会变成 JSON，这里以 `undefined` 为值就代表不会包含 key 对应的
          // 项。
          return [nsName, undefined!];
        }
        return [nsName, [...new Set(aliases)]];
      }),
    ),
    pagesByNamespace: Object.fromEntries(
      [...opts.namespaceMap].map(([nsName, ns]) => {
        return [nsName, Object.keys(ns.pages)];
      }),
    ),
    navigations: [...opts.namespaceMap.values()].map((ns) => ns.navigation),
  };
  await fs.writeFile(
    path.join(opts.output, "index.json"),
    JSON.stringify(index),
  );
}

await main(process.argv.slice(2));
