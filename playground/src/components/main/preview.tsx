import "./preview.scss";

import {
  Component,
  createEffect,
  createSignal,
  onMount,
  Setter,
  untrack,
} from "solid-js";

import { classModule, init, styleModule, type VNode } from "snabbdom";

import { parse } from "@rotext-lite/renderer-snabbdom";

const Preview: Component<
  {
    code: string;
    class?: string;
    setParsingTimeText: Setter<string>;
    onThrowInParsing: (thrown: unknown) => void;
  }
> = (props) => {
  let outputEl: HTMLDivElement;
  let patch: ReturnType<typeof init>;
  let lastNode: HTMLElement | VNode;

  const [errParse, setErrParse] = createSignal<unknown>(null);

  onMount(() => {
    patch = init(
      [classModule, styleModule],
      undefined,
      { experimental: { fragments: true } },
    );
    lastNode = outputEl;
  });

  createEffect(() => {
    try {
      if (untrack(() => errParse()) !== null) {
        setErrParse(null);
      }

      const parsingStart = performance.now();
      const vNode = parse(props.code, { breaks: true });
      props.setParsingTimeText(
        `${+(performance.now() - parsingStart).toFixed(3)}ms`,
      );

      patch(lastNode, vNode);
      lastNode = vNode;
    } catch (e) {
      setErrParse(e);
    }
  });

  return (
    <div
      class={`${props.class ?? ""} break-all prose previewer overflow-y-auto`}
    >
      <div ref={outputEl} />
    </div>
  );
};
export default Preview;
