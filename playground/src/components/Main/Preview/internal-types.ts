export interface LocationData {
  start: { line: number };
  end: { line: number };
}
export interface ElementLocationPair {
  element: HTMLElement;
  location: LocationData;
  offsetTop: number;
}
export type LookupList = ElementLocationPair[];
export type LookupListRaw = Omit<ElementLocationPair, "offsetTop">[];

/**
 * baseline 所穿过的元素、到达下一个这样的元素的进度，以及这个元素对应于原始输入的行数。
 */
export interface ScrollLocal {
  indexInLookupList: number;
  progress: number;
}
