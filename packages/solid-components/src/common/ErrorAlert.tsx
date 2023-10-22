import { Component, Show } from "solid-js";

import { createStyleProviderFromCSSText } from "@rotext/web-utils";

import { ShadowRootAttacher } from "../internal/mod";

import styles from "./ErrorAlert.scss?inline";

const styleProvider = createStyleProviderFromCSSText(styles);

const ErrorAlert: Component<{
  message?: string;
  stack?: string;
}> = (props) => {
  return (
    <ShadowRootAttacher styleProviders={[styleProvider]}>
      <div class="error-alert">
        <div class="container">
          <code class="message-area">
            {props.message}
            <Show when={props.stack}>
              <hr />
              {props.stack}
            </Show>
          </code>
        </div>
      </div>
    </ShadowRootAttacher>
  );
};

export default ErrorAlert;
