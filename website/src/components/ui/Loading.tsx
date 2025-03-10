import { Component } from "solid-js";

import {
  createStyleProviderFromCSSText,
  ShadowRootAttacher,
} from "@rolludejo/internal-web-shared/shadow-root";

import styles from "./Loading.css?inline";

const styleProvider = createStyleProviderFromCSSText(styles);

const Loading: Component = () => (
  <ShadowRootAttacher styleProviders={[styleProvider]}>
    <span class="loading-indicator"></span>
  </ShadowRootAttacher>
);

export default Loading;
