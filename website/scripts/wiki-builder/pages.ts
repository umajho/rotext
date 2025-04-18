import { parse as parseHTML } from "node-html-parser";
import { HtmlValidate } from "html-validate";

import { extractHeadings, mustParseAndRenderRotext } from "./utils";

const htmlValidator = new HtmlValidate({
  rules: {
    "prefer-tbody": "off",
  },
});

export interface Page {
  headings: Set<string>;
  wikiLinkTargets: Set<string>;

  html: string;
}

export function buildPage(
  pageText: string,
  opts: {
    /**
     * Note: 不是 `fullName`，因此不含命名空间。
     */
    name: string;
    fullName: string;
  },
): ["ok", Page] | ["error", string] {
  const html = mustParseAndRenderRotext(pageText);
  const dom = parseHTML(html);

  const headingsResult = extractHeadings(dom, {
    allowsOtherElements: true,
    ensuresHierarchy: true,
    allowedContent: {
      singleTextNode: true,
      singleWikiLink: false,
    },
  });
  if (headingsResult[0] === "error") {
    return ["error", `failed to collecting headings: ${headingsResult[1]}`];
  }
  const headings = headingsResult[1];
  const headingSet = new Set<string>();
  let h1Content!: string;
  for (const [i, h] of headings.entries()) {
    const heading = h.child.rawText.trim();
    if (headingSet.has(heading)) {
      return [
        "error",
        `duplicate heading: ` +
        `${JSON.stringify(heading)} (in ${JSON.stringify(opts.fullName)})`,
      ];
    }
    headingSet.add(heading);
    if (i === 0) {
      h1Content = heading;
    }
  }

  {
    const subPageName = getSubPageName(opts.name);

    if (h1Content !== subPageName) {
      return [
        "error",
        `sub page name ("${subPageName}") does not match \`h1\` content ("${h1Content}")`,
      ];
    }
  }

  for (const node of dom.querySelectorAll("x-rotext-example")) {
    const report = htmlValidator.validateStringSync(
      node.getAttribute("expected")!,
    );
    if (report.results.length) {
      return [
        "error",
        `invalid HTML in example: ${JSON.stringify(report.results)}`,
      ];
    }
  }

  const wikiLinkTargets = dom.querySelectorAll("x-wiki-link")
    .map((el) => el.getAttribute("address")!);

  return ["ok", {
    headings: new Set(headingSet),
    wikiLinkTargets: new Set(wikiLinkTargets),

    html,
  }];
}

function getSubPageName(name: string): string {
  return name.split("/").at(-1)!;
}
