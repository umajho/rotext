const PATH_PREFIX = import.meta.env.BASE_URL +
  "static/generated/syntax-reference";
const PATHS = {
  NAVIGATION: `${PATH_PREFIX}/navigation.json`,
  FILES_HEADINGS: `${PATH_PREFIX}/files-headings.json`,
  page: (pageName: string) => `${PATH_PREFIX}/${pageName}.inc.html`,
};

export interface Navigation {
  isPlain?: true;
  name: string;
  realName?: string;
  children?: Navigation[];
}

type PageToHeadingsMap = { [pageName: string]: string[] };
type HeadingToPageMap = { [heading: string]: string };

function createSyntaxReferenceResourceManager() {
  async function fetchNavigation() {
    return await (await fetch(PATHS.NAVIGATION)).json();
  }
  let navigationCache: Navigation | Promise<Navigation> | undefined;
  async function getNavigation(): Promise<Navigation> {
    return navigationCache ??= fetchNavigation()
      .then((v) => navigationCache = v);
  }

  async function fetchPageToHeadingsMap(): Promise<PageToHeadingsMap> {
    return await (await fetch(PATHS.FILES_HEADINGS)).json();
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
    return (await fetch(PATHS.page(pageName))).text();
  }
  const pageCaches: { [pageName: string]: string | Promise<string> } = {};
  async function getPage(pageName: string): Promise<string | null> {
    const map = await getPageToHeadingsMap();
    if (!(pageName in map)) return null;
    return pageCaches[pageName] ??= fetchPage(pageName)
      .then((v) => pageCaches[pageName] = v);
  }

  return { getNavigation, getPageToHeadingsMap, getHeadingToPageMap, getPage };
}

export const syntaxReferenceResourceManager =
  createSyntaxReferenceResourceManager();
