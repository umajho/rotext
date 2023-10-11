export function binarySearch<T>(
  list: Array<T>,
  predicate: (item: T, i: number) => true | "greater" | "less",
): number | null {
  let l = 0, h = list.length - 1;

  while (true) {
    if (h < l) return null;
    const i = ((h - l + 1) >> 2) + l;
    const item = list[i]!;
    const p = predicate(item, i);
    if (p === true) return i;
    if (p === "greater") {
      l = i + 1;
    } else {
      h = i - 1;
    }
  }
}
