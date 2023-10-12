export interface WidgetOwner {
  proseClass: string;
  widgetAnchorElement: () => HTMLElement;
  layoutChange: () => void;
  /**
   * 所属层级，`1` 为最顶层。
   */
  level: number;
}

const previewerElToWidgetOwnerMap = new WeakMap<HTMLElement, WidgetOwner>();

export function registerWidgetOwner(
  previewerEl: HTMLElement,
  owner: WidgetOwner,
) {
  previewerElToWidgetOwnerMap.set(previewerEl, owner);
}

export function getWidgetOwner(
  previewerEl: HTMLElement,
): WidgetOwner | undefined {
  return previewerElToWidgetOwnerMap.get(previewerEl);
}
