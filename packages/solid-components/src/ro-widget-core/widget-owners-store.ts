export interface WidgetOwner {
  widgetAnchorElement: HTMLElement;
  /**
   * 所属层级，`1` 为最顶层。
   */
  level: number;
  onLayoutChange: (listener: () => void) => void;
}

export interface WidgetOwnerController {
  nofityLayoutChange: () => void;
}

const previewerElToWidgetOwnerMap = new WeakMap<HTMLElement, WidgetOwner>();

export interface RegisterWidgetOwnerOptions {
  widgetAnchorElement: HTMLElement;
  level: number;
}

export function registerWidgetOwner(
  previewerEl: HTMLElement,
  opts: RegisterWidgetOwnerOptions,
): WidgetOwnerController {
  const layoutChangeListeners: (() => void)[] = [];

  const owner: WidgetOwner = {
    widgetAnchorElement: opts.widgetAnchorElement,
    level: opts.level,
    onLayoutChange: (l) => layoutChangeListeners.push(l),
  };
  previewerElToWidgetOwnerMap.set(previewerEl, owner);

  const controller: WidgetOwnerController = {
    nofityLayoutChange: () => layoutChangeListeners.forEach((l) => l()),
  };

  return controller;
}

export function getWidgetOwner(
  previewerEl: HTMLElement,
): WidgetOwner | undefined {
  return previewerElToWidgetOwnerMap.get(previewerEl);
}
