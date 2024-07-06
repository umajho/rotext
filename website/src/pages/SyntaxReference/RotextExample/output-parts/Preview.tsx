import { Component, onMount } from "solid-js";

import { registerRoWidgetOwner } from "@rotext/solid-components/internal";

import {
  PROSE_CLASS,
  WIDGET_OWNER_CLASS,
} from "../../../../utils/custom-elements-registration/mod";


export const Preview: Component<{ html: string }> = (props) => {
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

    outputWrapperEl.innerHTML = props.html;
  });

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
    </div>
  );
};
