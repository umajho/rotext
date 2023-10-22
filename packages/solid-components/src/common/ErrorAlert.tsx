import { Component, Show } from "solid-js";

import { createStyleProviderFromCSSText } from "@rotext/web-utils";

import { ShadowRootAttacher } from "../internal/mod";

import styles from "./ErrorAlert.scss?inline";

const styleProvider = createStyleProviderFromCSSText(styles);

const ErrorAlert: Component<{
  error: Error;
  showsStack: boolean;
}> = (props) => {
  return (
    <ShadowRootAttacher styleProviders={[styleProvider]}>
      <div class="error-alert">
        <div class="container">
          <code class="message-area">
            {props.error.message}
            <Show when={props.showsStack && props.error["stack"]}>
              <hr />
              {props.error["stack"]}
            </Show>
          </code>
        </div>
      </div>
    </ShadowRootAttacher>
  );
};

export default ErrorAlert;
