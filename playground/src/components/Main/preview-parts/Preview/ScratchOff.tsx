import { customElement, getCurrentElement, noShadowDOM } from "solid-element";
import { Component, onMount } from "solid-js";

export interface CreateScratchOffComponentOptions {
  innerNoAutoOpenClass: string;
}

function createScratchOffComponent(
  opts: CreateScratchOffComponentOptions & { tagName: string },
): Component {
  return () => {
    let dummyEl!: HTMLSpanElement;

    let currentElement = getCurrentElement();
    if (currentElement) {
      noShadowDOM();
    }

    const customEl = currentElement.closest(opts.tagName)!;

    customEl.classList.add(opts.innerNoAutoOpenClass);

    function handleClick() {
      customEl.classList.add("revealed");
      customEl.removeEventListener("click", handleClick);
    }
    customEl.addEventListener("click", handleClick);
    // NOTE: 当浏览器不支持使用自定义元素时，`<scratch-off />` 将不会有 `.by-click`。
    //       这时，揭示黑幕的逻辑将变为鼠标悬浮。
    customEl.classList.add("by-click");

    onMount(() => {
      dummyEl.outerText = "";
    });

    // NOTE: 如果返回空元素（`<></>` 之类的），则在编辑器中粘贴对应文本后，
    //       预览中对应新增的本元素不知为何，其内部为空。
    //       这里通过放置一个 dummy 元素，并在挂在时清除该元素来 workaround。
    return <span ref={dummyEl} class="hidden" />;
  };
}

export function registerCustomElement(
  tag: string,
  opts: CreateScratchOffComponentOptions,
) {
  customElement(tag, {}, createScratchOffComponent({ ...opts, tagName: tag }));
}
