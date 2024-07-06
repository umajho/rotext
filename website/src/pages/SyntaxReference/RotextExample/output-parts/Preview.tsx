import { Component, onMount } from "solid-js";

export const Preview: Component<{ html: string }> = (props) => {
  let el!: HTMLDivElement;

  onMount(() => {
    el.innerHTML = props.html;
  });

  return <div ref={el} />;
};
