/**
 * 为数组 `arr` 的每两个相邻元素之间插入一个 `sep`。
 */
export function intersperse<T>(arr: T[], sep: T): T[] {
  const result: T[] = Array(arr.length * 2 - 1);
  for (const [i, el] of arr.entries()) {
    result[i * 2] = el;
    if (i !== arr.length - 1) {
      result[i * 2 + 1] = sep;
    }
  }
  return result;
}

/**
 * 为数组 `arr` 的每两个相邻元素之间插入一个 `sepFactory` 返回的内容。
 */
export function intersperseWithFactory<T>(arr: T[], sepFactory: () => T): T[] {
  const result: T[] = Array(arr.length * 2 - 1);
  for (const [i, el] of arr.entries()) {
    result[i * 2] = el;
    if (i !== arr.length - 1) {
      result[i * 2 + 1] = sepFactory();
    }
  }
  return result;
}
