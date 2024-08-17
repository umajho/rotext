async function fetchExample(fileName: string): Promise<string> {
  return await (await fetch(
    import.meta.env.BASE_URL + "static/docs/" + fileName,
  )).text();
}

const source = {
  "入门": () => fetchExample("rotext入门.rotext"),
  "入门-legacy": () => fetchExample("rotext入门-legacy.rotext"),
};

type Names = keyof typeof source;

const cache: { [name in Names]?: string | Promise<string> } = {};

export default {
  keys: (): Names[] => {
    return Object.keys(source) as Names[];
  },

  get: (name: Names) => {
    if (name in cache) return cache[name] as string | Promise<string>;

    return cache[name] = source[name]()!.then(
      (text) => {
        cache[name] = text;
        return text;
      },
    );
  },
};
