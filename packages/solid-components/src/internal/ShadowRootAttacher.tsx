import { Component, JSX, onMount } from "solid-js";
import { render } from "solid-js/web";

import { adoptStyle, StyleProvider } from "@rotext/web-utils";

const ShadowRootAttacher: Component<{
  mode?: ShadowRootMode;
  styleProviders?: StyleProvider[];
  hostStyle?: string | JSX.CSSProperties;

  children: JSX.Element;
}> = (props) => {
  let hostEl!: HTMLDivElement;

  onMount(() => {
    const shadowRoot = hostEl.attachShadow({ mode: props.mode ?? "open" });

    if (props.styleProviders) {
      for (const p of props.styleProviders) {
        adoptStyle(shadowRoot, p);
      }
    }

    render(() => props.children, shadowRoot);
  });

  return <div ref={hostEl} style={props.hostStyle} />;
};

export default ShadowRootAttacher;
