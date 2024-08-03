async function fetchExample(fileName: string): Promise<string> {
  return await (await fetch(
    import.meta.env.BASE_URL + "static/docs/" + fileName,
  )).text();
}

const examples = {
  "入门": fetchExample("rotext入门.rotext"),
  "入门-legacy": fetchExample("rotext入门-legacy.rotext"),
};

const cache: { [name in keyof typeof examples]?: string | Promise<string> } =
  {};

const exampleProxy = new Proxy(examples, {
  get(target, prop_): string | Promise<string> {
    const prop = prop_ as keyof typeof target;

    if (prop in cache) return cache[prop] as string | Promise<string>;

    return cache[prop] = target[prop]!.then(
      (text) => {
        cache[prop] = text;
        return text;
      },
    );
  },
}) as { [name in keyof typeof examples]: string | Promise<string> };

export default exampleProxy;
