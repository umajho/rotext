import { Component, Show } from "solid-js";

import { createStyleProviderFromCSSText } from "@rotext/web-utils";

import { ShadowRootAttacher } from "../internal/mod";

import styles from "./ErrorAlert.scss?inline";

const styleProvider = createStyleProviderFromCSSText(styles);

const ErrorAlert: Component<
  & { kindText?: string }
  & ({ message: string; stack?: string } | { message: undefined })
> = (props) => {
  return (
    <ShadowRootAttacher styleProviders={[styleProvider]}>
      <div class="error-alert">
        <div class="heading">{props.kindText}错误</div>
        <Show when={"message" in props}>
          <code class="message-area">
            {(props as any).message}
            <Show when={"stack" in props}>
              <hr />
              {(props as any).stack}
            </Show>
          </code>
        </Show>
      </div>
    </ShadowRootAttacher>
  );
};

export default ErrorAlert;
