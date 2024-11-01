const PATH_PREFIX = import.meta.env.BASE_URL +
  "static/generated/syntax-reference";
const PATHS = {
  NAVIGATION: `${PATH_PREFIX}/navigation.json`,
  /**
   * TODO!!: 目前只剩下被 `getPage` 拿来判断页面是否存在的功能，实际可以移除掉。
   */
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

  return { getNavigation, getPageToHeadingsMap, getPage };
}

export const syntaxReferenceResourceManager =
  createSyntaxReferenceResourceManager();
