import { LookupList } from "../components/Main/Preview/internal-types";

interface Previewer {
  widgetAnchorElement: () => HTMLElement;
  /**
   * NOTE: 由于元素位置更新时（由代码变更或调整滚动容器大小造成）lookupList 也会随之更新，
   *       即使不需要其中的数据，也可以用来检测是否需要重新计算位置等信息。
   */
  lookupList: () => LookupList;
  /**
   * 所属层级，`1` 为最顶层。
   */
  level: number;
}

const previewMap = new WeakMap<HTMLElement, Previewer>();

export function registerPreviewer(
  previewerEl: HTMLElement,
  previewer: Previewer,
) {
  previewMap.set(previewerEl, previewer);
}

export function getPreviewer(previewerEl: HTMLElement): Previewer | undefined {
  return previewMap.get(previewerEl);
}
