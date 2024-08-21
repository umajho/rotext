export interface WidgetOwner {
  popperAnchorElement: HTMLElement;
  /**
   * 所属层级，`1` 为最顶层。
   */
  level: number;
  layoutChangeObserver: LayoutChangeObserver;
}

export interface WidgetOwnerController {}

export interface LayoutChangeObserver {
  subscribe(cb: () => void): void;
  unsubscribe(cb: () => void): void;
}

const previewerElToWidgetOwnerMap = new WeakMap<HTMLElement, WidgetOwner>();

export interface RegisterWidgetOwnerOptions {
  popperAnchorElement: HTMLElement;
  level: number;
  layoutChangeObserver: LayoutChangeObserver;
}

export function registerWidgetOwner(
  ownerEl: HTMLElement,
  opts: RegisterWidgetOwnerOptions,
): WidgetOwnerController {
  const owner: WidgetOwner = Object.freeze({ ...opts });
  previewerElToWidgetOwnerMap.set(ownerEl, owner);

  const controller: WidgetOwnerController = {};

  return controller;
}

export function getWidgetOwner(
  previewerEl: HTMLElement,
): WidgetOwner | undefined {
  return previewerElToWidgetOwnerMap.get(previewerEl);
}
