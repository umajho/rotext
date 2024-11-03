const PATH_PREFIX = import.meta.env.BASE_URL +
  "static/generated/wiki";
const PATHS = {
  NAVIGATION: `${PATH_PREFIX}/navigation.json`,
  INDEX: `${PATH_PREFIX}/index.json`,
  page: (pageName: string) => `${PATH_PREFIX}/${pageName}.inc.html`,
};

export interface Index {
  namespaceAliases: { [k: string]: string[] };
  pagesByNamespace: { [k: string]: string[] };
  navigations: Navigation[];
}

export interface Navigation {
  isPlain?: true;
  name: string;
  realName?: string;
  children?: Navigation[];
}

const index: Index = await (await fetch(PATHS.INDEX)).json();
const namespaceAliasMap = (() => {
  const m: Record<string, string> = {};
  for (const nsName of Object.keys(index.pagesByNamespace)) {
    m[nsName] = nsName;
    if (nsName in index.namespaceAliases) {
      for (const alias of index.namespaceAliases[nsName]!) {
        m[alias] = nsName;
      }
    }
  }
  return m;
})();
const authenticFullPageNames = (() => {
  const ret: string[] = [];
  for (const [nsName, names] of Object.entries(index.pagesByNamespace)) {
    for (const name of names) {
      ret.push(`${nsName}:${name}`);
    }
  }
  return new Set(ret);
})();

function getNavigations(): Navigation[] {
  return index.navigations;
}

async function fetchPage(fullPageName: string): Promise<string> {
  const [ns, pageName] = fullPageName.split(":");
  return (await fetch(PATHS.page(`${ns}/${pageName}`))).text();
}
const pageCaches: {
  [fullPageName: string]: DocumentFragment | Promise<DocumentFragment>;
} = {};
async function getPage(fullPageName: string): Promise<DocumentFragment | null> {
  const authenticFullPageName = getAuthenticFullPageName(fullPageName);
  if (!authenticFullPageName) return null;

  if (!(authenticFullPageNames.has(authenticFullPageName))) {
    return null;
  }
  return pageCaches[authenticFullPageName] ??= fetchPage(authenticFullPageName)
    .then((v) => {
      const tEl = document.createElement("template");
      tEl.innerHTML = v;
      return pageCaches[authenticFullPageName] = tEl.content;
    });
}

function getAuthenticFullPageName(
  fullPageName: string,
): string | null {
  let [ns, name] = fullPageName.split(":", 2);
  if (name === undefined) {
    [ns, name] = ["main", ns];
  } else if (!(ns! in namespaceAliasMap)) {
    return null;
  } else {
    ns = namespaceAliasMap[ns!];
  }
  return `${ns}:${name}`;
}

export const wikiResourceManager = {
  getNavigations,
  getPage,
  getAuthenticFullPageName,
};
