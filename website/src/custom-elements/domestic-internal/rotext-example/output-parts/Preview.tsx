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

import * as Ankor from "ankor";

import { PROSE_CLASS } from "../../../mod";

export type PreviewContent = ["html", string];

export const Preview: Component<{
  content: () => PreviewContent;
}> = (props) => {
  let containerEl!: HTMLDivElement;
  let outputWrapperEl!: HTMLDivElement;

  const [isOutputEmpty, setIsOutputEmpty] = createSignal(false);
  onMount(() => {
    //==== 文档渲染 ====
    setUpRendering({
      content: props.content,
      els: { outputWrapper: outputWrapperEl },
      setIsOutputEmpty,
    });
  });

  const widgetOwnerData = JSON.stringify(
    {
      level: 1,
    } satisfies Ankor.WidgetOwnerRaw,
  );

  return (
    <div
      ref={containerEl}
      class={[
        Ankor.WIDGET_OWNER_CLASS,
        "relative h-full p-4",
        "tuan-background",
      ].join(" ")}
      data-ankor-widget-owner={widgetOwnerData}
    >
      <div class={`${Ankor.ANCHOR_CLASS} relative z-10`} />
      <div
        class={[
          Ankor.CONTENT_CLASS,
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
