import { Component } from "solid-js";
import { customElement, noShadowDOM } from "solid-element";

import { ShadowRootAttacher } from "@rolludejo/web-internal";

import { styleProvider as styleProviderForPreflight } from "../../../styles/preflight";
import { styleProvider as styleProviderForTailwind } from "../../../styles/tailwind";
import { styleProvider as styleProviderForOneDark } from "../../../styles/one-dark";

import { createRotextExampleStore } from "./store";

import { MainCard } from "./MainCard";

export function registerCustomElement(tag: string) {
  customElement(tag, { input: "", expected: null }, RotextExample);
}

// TODO!!: 改成 widget 那样挂在 widget anchor 上。
export const RotextExample: Component<
  { input: string; expected: string | null }
> = (props) => {
  noShadowDOM();

  const store = createRotextExampleStore({
    originalInput: props.input,
    originalExpected: props.expected ?? "",
  });

  return (
    <ShadowRootAttacher
      styleProviders={[
        styleProviderForPreflight,
        styleProviderForTailwind,
        styleProviderForOneDark,
      ]}
    >
      <MainCard store={store} />
    </ShadowRootAttacher>
  );
};
