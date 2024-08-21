import {
  Component,
  createEffect,
  createSignal,
  on,
  onMount,
  Show,
} from "solid-js";

// @ts-ignore
import { Idiomorph } from "idiomorph/dist/idiomorph.esm.js";

import {
  ElementLayoutChangeObserver,
  registerRoWidgetOwner,
} from "@rotext/solid-components/internal";

import {
  PROSE_CLASS,
  registerCustomElementsOnce,
  WIDGET_OWNER_CLASS,
} from "../../../../utils/custom-elements-registration/mod";

export type PreviewContent = ["html", string];

registerCustomElementsOnce();

export const Preview: Component<{
  content: () => PreviewContent;
}> = (props) => {
  let containerEl!: HTMLDivElement;
  let popperAnchorEl!: HTMLDivElement;
  let outputWrapperEl!: HTMLDivElement;

  const [isOutputEmpty, setIsOutputEmpty] = createSignal(false);
  onMount(() => {
    //==== 注册进全局存储 ====
    registerRoWidgetOwner(containerEl, {
      popperAnchorElement: popperAnchorEl,
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
      <div class="relative z-10" ref={popperAnchorEl} />
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
  createEffect(on([opts.content], ([content]) => {
    if (content[0] === "html") {
      Idiomorph.morph(opts.els.outputWrapper, content[1], {
        morphStyle: "innerHTML",
      });
      opts.setIsOutputEmpty(!content[1]);
    } else {
      throw new Error("unreachable");
    }
  }));
}
