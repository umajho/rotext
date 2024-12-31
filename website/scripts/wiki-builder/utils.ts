import { type HTMLElement, type Node } from "node-html-parser";

import { TAG_NAME_MAP } from "../../src/custom-elements/consts";
import { Navigation } from "../../src/resource-managers/wiki";

import { rotextAdapter } from "./global";
import { NamespaceMap } from "./namespaces";

export function mustParseAndRenderRotext(input: string) {
  const result = rotextAdapter.parseAndRender(input, {
    tagNameMap: TAG_NAME_MAP,
    shouldIncludeBlockIDs: false,
  });
  if (result[0] !== "ok") {
    throw new Error("failed to parse and render rotext: " + result[1]);
  }
  result[0] satisfies "ok";
  return result[1].html;
}

interface Heading {
  level: number;
  child: Node;
}

export function extractHeadings(dom: HTMLElement, opts: {
  allowsOtherElements: boolean;
  ensuresHierarchy: true;
  allowedContent: {
    singleTextNode: true;
    singleWikiLink: boolean;
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
      opts.allowedContent.singleWikiLink &&
      child.rawTagName === "x-wiki-link"
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

export type NamespaceAliasMap =
  & Map<string, string>
  & { readonly __tag: unique symbol };

export function extractNamespaceAliasMap(
  nsMap: NamespaceMap,
): NamespaceAliasMap {
  const nsAliasMap = new Map() as NamespaceAliasMap;

  for (const [nsName, ns] of nsMap) {
    nsAliasMap.set(nsName, nsName);
    for (const alias of ns.configuration.namespace.aliases ?? []) {
      nsAliasMap.set(alias, nsName);
    }
  }

  return nsAliasMap as NamespaceAliasMap;
}

export type FullPageNameToHeadingsMap =
  & Map<string, Set<string>>
  & { readonly __tag: unique symbol };

export function collectAuthenticFullPageNameToHeadingsMap(
  nsMap: NamespaceMap,
): FullPageNameToHeadingsMap {
  const map = new Map() as FullPageNameToHeadingsMap;

  for (const [nsName, ns] of nsMap) {
    for (const [pageName, page] of Object.entries(ns.pages)) {
      map.set(`${nsName}:${pageName}`, page.headings);
    }
  }

  return map as FullPageNameToHeadingsMap;
}

export function getAuthenticFullPageName(
  fullPageName: string,
  nsAliasMap: NamespaceAliasMap,
): ["ok", string] | ["error", string] {
  let [ns, name] = fullPageName.split(":", 2);
  if (name === undefined) {
    [ns, name] = ["main", ns];
  } else if (!nsAliasMap.has(ns!)) {
    return ["error", `unknown namespace ${JSON.stringify(ns)}`];
  } else {
    ns = nsAliasMap.get(ns!);
  }
  return ["ok", `${ns}:${name}`];
}

export function extractTargetsInNavigation(
  navigation: Navigation,
  nsAliasMap: NamespaceAliasMap,
): Set<string> {
  const targets = new Set<string>();
  doIt(targets, navigation, nsAliasMap);
  return targets;

  function doIt(
    targets: Set<string>,
    navigation: Navigation,
    nsAliasMap: NamespaceAliasMap,
  ) {
    if (!navigation.isPlain) {
      const name = navigation.realName ?? navigation.name;
      targets.add(name);
    }

    for (const child of navigation.children ?? []) {
      doIt(targets, child, nsAliasMap);
    }
  }
}
