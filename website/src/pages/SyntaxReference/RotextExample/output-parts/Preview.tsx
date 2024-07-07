import {
  Component,
  createEffect,
  createSignal,
  on,
  onMount,
  Show,
} from "solid-js";

import { registerRoWidgetOwner } from "@rotext/solid-components/internal";

import {
  PROSE_CLASS,
  registerCustomElementsOnce,
  WIDGET_OWNER_CLASS,
} from "../../../../utils/custom-elements-registration/mod";

registerCustomElementsOnce();

export const Preview: Component<{ content: ["html", string] }> = (props) => {
  let containerEl!: HTMLDivElement;
  let widgetAnchorEl!: HTMLDivElement;
  let outputWrapperEl!: HTMLDivElement;

  onMount(() => {
    const cbs = new Set<() => void>();
    const layoutChangeObserver = {
      subscribe: (cb: () => void) => cbs.add(cb),
      unsubscribe: (cb: () => void) => cbs.delete(cb),
    };
    registerRoWidgetOwner(containerEl, {
      widgetAnchorElement: widgetAnchorEl,
      level: 1,
      layoutChangeObserver,
    });
  });

  const [isOutputEmpty, setIsOutputEmpty] = createSignal(false);
  createEffect(on([() => props.content], ([content]) => {
    if (content[0] === "html") {
      outputWrapperEl.innerHTML = content[1];
      setIsOutputEmpty(!content[1]);
    }
  }));

  return (
    <div
      class={[
        WIDGET_OWNER_CLASS,
        "relative h-full p-4",
        "tuan-background",
      ].join(" ")}
      ref={containerEl}
    >
      <div ref={widgetAnchorEl} />
      <div
        class={[
          "relative",
          "self-center mx-auto",
          "break-all",
          PROSE_CLASS,
        ].join(" ")}
      >
        <div ref={outputWrapperEl} />
      </div>
      <Show when={isOutputEmpty()}>
        <div class="w-full flex justify-center">
          <div class="text-gray-400">
            （输出为空…）
          </div>
        </div>
      </Show>
    </div>
  );
};
