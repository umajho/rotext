import { ANCHOR_CLASS, CONTENT_CLASS, WIDGET_OWNER_CLASS } from "./consts";

interface WidgetOwner {
  anchorElement: HTMLElement;
  level: number;
  layoutChangeObserver: LayoutChangeObserver;

  controller: WidgetOwnerController;
}

/**
 * 供外部使用。
 */
export interface WidgetOwnerController {
  notifyLayoutChange(): void;
}

export interface WidgetOwnerRaw {
  /**
   * 所属层级，`1` 为最顶层。
   */
  level: number;
}

const elementToWidgetOwnerMap = new WeakMap<Node, WidgetOwner>();

function initializeAgentOwner(widgetOwnerEl: HTMLElement): WidgetOwner {
  if (!widgetOwnerEl.classList.contains(WIDGET_OWNER_CLASS)) {
    console.error("not widget owner", { widgetOwner: widgetOwnerEl });
    throw new Error("not widget owner");
  }

  let wo = elementToWidgetOwnerMap.get(widgetOwnerEl);
  if (!wo) {
    const woRaw: WidgetOwnerRaw = //
      JSON.parse(widgetOwnerEl.dataset["ankorWidgetOwner"]!);

    const level = woRaw.level;
    if (!(Number.isInteger(level) && level > 0)) {
      console.error("bad level", { level });
      throw new Error("bad level");
    }

    const anchorSelector = `.${ANCHOR_CLASS}`;
    const anchorElement = //
      widgetOwnerEl.querySelector(anchorSelector) as HTMLElement;
    if (!anchorElement) {
      console.error("anchor not found", {
        widgetOwner: widgetOwnerEl,
        anchorSelector,
      });
      throw new Error("anchor not found");
    }

    const contentSelector = `.${CONTENT_CLASS}`;
    const contentElement = //
      widgetOwnerEl.querySelector(contentSelector) as HTMLElement;
    if (!contentElement) {
      console.error("content not found", {
        widgetOwner: widgetOwnerEl,
        contentSelector,
      });
      throw new Error("content not found");
    }
    const layoutChangeObserver = //
      createLayoutChangeObserver(contentElement);

    const controller: WidgetOwnerController = {
      notifyLayoutChange: layoutChangeObserver.notify,
    };

    wo = {
      anchorElement,
      level,
      layoutChangeObserver,
      controller,
    };

    elementToWidgetOwnerMap.set(widgetOwnerEl, wo);
  }

  return wo;
}

export type WidgetOwnerAgent = ReturnType<typeof createWidgetOwnerAgent>;
/**
 * 创建与所属 widget owner 沟通的代理。
 *
 * `elInside` 与 widget owner 之间不能隔着 shadow root。
 */
export function createWidgetOwnerAgent(elInside: HTMLElement) {
  const wo = initializeAgentOwner(elInside.closest(`.${WIDGET_OWNER_CLASS}`)!);

  return wo;
}

export function getWidgetOwnerController(
  widgetOwnerEl: HTMLElement,
): WidgetOwnerController | undefined {
  const wo = initializeAgentOwner(widgetOwnerEl);

  return wo.controller;
}

interface LayoutChangeObserver {
  subscribe(cb: () => void): void;
  unsubscribe(cb: () => void): void;

  notify(): void;
}

function createLayoutChangeObserver(el: HTMLElement): LayoutChangeObserver {
  const cbs = new Set<() => void>();
  function notify() {
    [...cbs].forEach((cb) => cb());
  }

  const contentResizeObserver = new ResizeObserver(notify);
  function subscribe(cb: () => void) {
    cbs.add(cb);
  }
  function unsubscribe(cb: () => void) {
    cbs.delete(cb);
  }
  contentResizeObserver.observe(el);

  return { subscribe, unsubscribe, notify };
}
