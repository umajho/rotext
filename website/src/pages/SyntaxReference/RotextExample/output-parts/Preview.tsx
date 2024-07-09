import {
  Component,
  createEffect,
  createSignal,
  on,
  onMount,
  Show,
} from "solid-js";

import {
  attributesModule,
  Classes,
  classModule,
  h,
  init,
  styleModule,
  VNode,
  VNodeChildren,
} from "snabbdom";

import {
  ElementLayoutChangeObserver,
  registerRoWidgetOwner,
} from "@rotext/solid-components/internal";

import {
  PROSE_CLASS,
  registerCustomElementsOnce,
  WIDGET_OWNER_CLASS,
} from "../../../../utils/custom-elements-registration/mod";

export type PreviewContent =
  | ["html", string]
  | ["v-node-children", VNodeChildren];

registerCustomElementsOnce();

export const Preview: Component<{
  content: () => PreviewContent;
}> = (props) => {
  let containerEl!: HTMLDivElement;
  let widgetAnchorEl!: HTMLDivElement;
  let outputWrapperEl!: HTMLDivElement;

  const [isOutputEmpty, setIsOutputEmpty] = createSignal(false);
  onMount(() => {
    //==== 注册进全局存储 ====
    registerRoWidgetOwner(containerEl, {
      widgetAnchorElement: widgetAnchorEl,
      level: 1,
      layoutChangeObserver: new ElementLayoutChangeObserver(
        outputWrapperEl,
        { resize: true },
      ),
    });

    //==== 文档渲染 ====
    setUpRendering({
      content: props.content,
      els: { outputWrapper: outputWrapperEl },
      setIsOutputEmpty,
    });
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

function setUpRendering(opts: {
  content: () => PreviewContent;
  els: {
    outputWrapper: HTMLDivElement;
  };
  setIsOutputEmpty: (value: boolean) => void;
}) {
  let patch: ReturnType<typeof init> | null = null;
  let lastNode: HTMLElement | VNode | null = null;
  createEffect(on([opts.content], ([content]) => {
    if (content[0] === "html") {
      patch = null;
      lastNode = null;

      opts.els.outputWrapper.innerHTML = content[1];
      opts.setIsOutputEmpty(!content[1]);
    } else if (content[0] === "v-node-children") {
      if (!patch) {
        opts.els.outputWrapper.innerText = "";
        const outputEl = document.createElement("div");
        opts.els.outputWrapper.appendChild(outputEl);

        patch = init(
          [classModule, styleModule, attributesModule],
          undefined,
          { experimental: { fragments: true } },
        );
        lastNode = outputEl;
      }

      const classMap: Classes = { "relative": true };
      const vNode = h("article", { class: classMap }, content[1]);

      patch(lastNode!, vNode);
      lastNode = vNode;
    } else {
      throw new Error("unreachable");
    }
  }));
}
