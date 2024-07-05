import { Component } from "solid-js";
import { customElement, noShadowDOM } from "solid-element";

import { ShadowRootAttacher } from "@rolludejo/web-internal";

import { Card } from "../../../components/ui/mod";

import { styleProvider as styleProviderForPreflight } from "../../../styles/preflight";
import { styleProvider as styleProviderForTailwind } from "../../../styles/tailwind";
import { styleProvider as styleProviderForOneDark } from "../../../styles/one-dark";

export function registerCustomElement(tag: string) {
  customElement(tag, { input: "", expected: null }, RotextExample);
}

// TODO!!: 改成 widget 那样挂在 widget anchor 上。
export const RotextExample: Component<
  { input: string; expected: string | null }
> = (props) => {
  noShadowDOM();

  return (
    <ShadowRootAttacher
      styleProviders={[
        styleProviderForPreflight,
        styleProviderForTailwind,
        styleProviderForOneDark,
      ]}
    >
      <div class="relative">
        <div class="absolute z-10 left-4">
          <div class="bg-indigo-800 px-8 py-2 rounded-lg">
            预览
          </div>
        </div>
      </div>
      <div class="px-4 py-6">
        <Card class="bg-indigo-800">
          <div class="grid grid-cols-1 xl:grid-cols-2">
            <pre class="one-dark one-dark-background px-4 py-2">
                <code>{props.input}</code>
            </pre>
            <div class="bg-black overflow-y-scroll">
              <pre class="px-4 py-2">
                <code>{props.expected}</code>
              </pre>
            </div>
          </div>
          TODO!!!: empty, group, example-fixture (name), (preview).
        </Card>
      </div>
    </ShadowRootAttacher>
  );
};
