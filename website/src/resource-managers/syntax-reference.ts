const PATH_PREFIX = import.meta.env.BASE_URL +
  "static/generated/syntax-reference";
const FILES_HEADINGS_PATH = `${PATH_PREFIX}/files-headings.json`;

const makePagePath = (pageName: string) =>
  `${PATH_PREFIX}/${pageName}.inc.html`;

type PageToHeadingsMap = { [pageName: string]: string[] };
type HeadingToPageMap = { [heading: string]: string };

function createSyntaxReferenceResourceManager() {
  async function fetchPageToHeadingsMap(): Promise<PageToHeadingsMap> {
    return await (await fetch(FILES_HEADINGS_PATH)).json();
  }
  let pageToHeadingsMapCache:
    | PageToHeadingsMap
    | Promise<PageToHeadingsMap>
    | undefined;
  async function getPageToHeadingsMap(): Promise<PageToHeadingsMap> {
    return pageToHeadingsMapCache ??= fetchPageToHeadingsMap()
      .then((v) => pageToHeadingsMapCache = v);
  }

  function makeHeadingToPageMap(map: PageToHeadingsMap): HeadingToPageMap {
    const index: HeadingToPageMap = {};
    for (const [filePath, headings] of Object.entries(map)) {
      for (const heading of headings) {
        index[heading] = filePath;
      }
    }
    return index;
  }
  let headingToPageMapCache:
    | HeadingToPageMap
    | Promise<HeadingToPageMap>
    | undefined;
  async function getHeadingToPageMap(): Promise<HeadingToPageMap> {
    return headingToPageMapCache ??= getPageToHeadingsMap()
      .then((v) => headingToPageMapCache = makeHeadingToPageMap(v));
  }

  async function fetchPage(pageName: string): Promise<string> {
    return (await fetch(makePagePath(pageName))).text();
  }
  const pageCaches: { [pageName: string]: string | Promise<string> } = {};
  async function getPage(pageName: string): Promise<string | null> {
    const map = await getPageToHeadingsMap();
    if (!(pageName in map)) return null;
    return pageCaches[pageName] ??= fetchPage(pageName)
      .then((v) => pageCaches[pageName] = v);
  }

  return { getPageToHeadingsMap, getHeadingToPageMap, getPage };
}

export const syntaxReferenceResourceManager =
  createSyntaxReferenceResourceManager();
