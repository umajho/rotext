const PATH_PREFIX = import.meta.env.BASE_URL +
  "static/generated/syntax-reference";
const PATHS = {
  NAVIGATION: `${PATH_PREFIX}/navigation.json`,
  PAGES_OF_SYNTAX_REFERENCES: `${PATH_PREFIX}/pages-of-syntax-references.json`,
  page: (pageName: string) => `${PATH_PREFIX}/${pageName}.inc.html`,
};

export interface Navigation {
  isPlain?: true;
  name: string;
  realName?: string;
  children?: Navigation[];
}

type Pages = Set<string>;

function createSyntaxReferenceResourceManager() {
  async function fetchNavigation() {
    return await (await fetch(PATHS.NAVIGATION)).json();
  }
  let navigationCache: Navigation | Promise<Navigation> | undefined;
  async function getNavigation(): Promise<Navigation> {
    return navigationCache ??= fetchNavigation()
      .then((v) => navigationCache = v);
  }

  async function fetchPagesOfSyntaxReferences(): Promise<Pages> {
    return new Set(
      await (await fetch(PATHS.PAGES_OF_SYNTAX_REFERENCES)).json(),
    );
  }
  let pageToHeadingsMapCache: Pages | Promise<Pages> | undefined;
  async function getPagesOfSyntaxReferences(): Promise<Pages> {
    return pageToHeadingsMapCache ??= fetchPagesOfSyntaxReferences()
      .then((v) => pageToHeadingsMapCache = v);
  }

  async function fetchPage(pageName: string): Promise<string> {
    return (await fetch(PATHS.page(pageName))).text();
  }
  const pageCaches: { [pageName: string]: string | Promise<string> } = {};
  async function getPage(pageName: string): Promise<string | null> {
    const pages = await getPagesOfSyntaxReferences();
    if (!(pages.has(pageName))) return null;
    return pageCaches[pageName] ??= fetchPage(pageName)
      .then((v) => pageCaches[pageName] = v);
  }

  return { getNavigation, getPage };
}

export const syntaxReferenceResourceManager =
  createSyntaxReferenceResourceManager();
