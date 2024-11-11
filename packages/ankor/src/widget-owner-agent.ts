import { ANCHOR_CLASS, CONTENT_CLASS, WIDGET_OWNER_CLASS } from "./consts";

type AddressInWidgetOwner =
  | ["reference" | "internal", string]
  | ["special", "live" | "never"];

interface WidgetOwner {
  rawData: string;

  anchorElement: HTMLElement;

  /**
   * 所属层级。详见 `WidgetOwnerRaw` 中同名属性的文档。
   *
   * 来源于属性 `data-ankor-widget-owner`。在且仅在每次创建（包括已创建过而直接
   * 返回创建过的实例的情况）WidgetOwnerAgent 时更新，该更新不存在通知方式。因此，
   * 更新某 WidgetOwner 的元素的前述属性之值的前置条件是：此时属于该 WidgetOwner
   * 的所有挂件都应将被清除。典型的情况：单页面应用在同一路由下从一个页面切换到
   * 另一个页面。
   */
  level: number;
  /**
   * 内容的地址。详见 `WidgetOwnerRaw` 中同名属性的文档。
   *
   * 来源于属性 `data-ankor-widget-owner`。有关更新的行为同 `level` 属性。
   */
  address: AddressInWidgetOwner;

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

  /**
   * 内容的地址，不含锚，可以不存在（赋予 `null`）。
   *
   * 元组的第一项代表地址的类型，可以为：
   * - `"reference"`：用于帖子。
   * - `"internal"`：用于 wiki 页面。
   */
  address: AddressInWidgetOwner;
}

const elementToWidgetOwnerMap = new WeakMap<Node, WidgetOwner>();

function initializeAgentOwner(widgetOwnerEl: HTMLElement): WidgetOwner {
  if (!widgetOwnerEl.classList.contains(WIDGET_OWNER_CLASS)) {
    console.error("not widget owner", { widgetOwner: widgetOwnerEl });
    throw new Error("not widget owner");
  }

  const currentRawData = widgetOwnerEl.dataset["ankorWidgetOwner"]!;

  let wo = elementToWidgetOwnerMap.get(widgetOwnerEl);
  if (wo && currentRawData !== wo.rawData) {
    const woRaw: WidgetOwnerRaw = JSON.parse(currentRawData);

    wo.level = mustBeValidLevel(woRaw.level);
    wo.address = woRaw.address;
  } else if (!wo) {
    const woRaw: WidgetOwnerRaw = JSON.parse(currentRawData);

    const level = mustBeValidLevel(woRaw.level);
    const address = woRaw.address;

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
      rawData: currentRawData,
      anchorElement,
      level,
      address,
      layoutChangeObserver,
      controller,
    };

    elementToWidgetOwnerMap.set(widgetOwnerEl, wo);
  }

  return wo;
}

/**
 * @throws
 */
function mustBeValidLevel(level: number): number {
  if (!(Number.isInteger(level) && level > 0)) {
    console.error("bad level", { level });
    throw new Error("bad level");
  }
  return level;
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
    for (const cb of cbs) {
      cb();
    }
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
