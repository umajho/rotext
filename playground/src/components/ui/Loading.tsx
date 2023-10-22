import { Component } from "solid-js";

import { createStyleProviderFromCSSText } from "@rotext/web-utils";
import { ShadowRootAttacher } from "@rotext/solid-components/internal";

import styles from "./Loading.scss?inline";

const styleProvider = createStyleProviderFromCSSText(styles);

const Loading: Component = () => (
  <ShadowRootAttacher styleProviders={[styleProvider]}>
    <span class="loading-indicator"></span>
  </ShadowRootAttacher>
);

export default Loading;
