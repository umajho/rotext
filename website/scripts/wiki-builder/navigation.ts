import { type HTMLElement, parse as parseHTML } from "node-html-parser";

import { Navigation } from "../../src/resource-managers/wiki";

import { extractHeadings, mustParseAndRenderRotext } from "./utils";

export function buildNavigation(
  navigationText: string,
): ["ok", Navigation] | ["error", string] {
  const html = mustParseAndRenderRotext(navigationText);
  const dom = parseHTML(html);

  const headingsResult = extractHeadings(dom, {
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
    const realName = (heading.child as HTMLElement).getAttribute?.("address");

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
