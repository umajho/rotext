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
 * 返回两个字符串的公共前缀。
 */
export function getCommonPrefix(a: string, b: string): string {
  let result = "";
  const minLen = Math.min(a.length, b.length);
  for (let i = 0; i < minLen; i++) {
    if (a[i] === b[i]) {
      result += a[i];
    } else {
      break;
    }
  }
  return result;
}
