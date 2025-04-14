import { customElement, getCurrentElement } from "solid-element";
import { Component, createMemo, onMount } from "solid-js";

import { adoptStyle } from "@rolludejo/internal-web-shared/shadow-root";

import { styleProvider as styleProviderForPreflight } from "../../../styles/preflight";
import { styleProvider as styleProviderForTailwind } from "../../../styles/tailwind";

import {
  BadParametersErrorMessage,
  ErrorMessage,
  getCallTypeTitle,
  Name,
} from "./shared";

function createBlockCallErrorComponent(): Component<{
  "call-type": "transclusion" | "extension" | "";
  "call-name": string;
  "error-type": string;
  "error-value": string | null;
}> {
  return (props) => {
    const currentElement = getCurrentElement();

    const title = createMemo(() => getCallTypeTitle(props["call-type"]));

    onMount(() => {
      for (const p of [styleProviderForPreflight, styleProviderForTailwind]) {
        adoptStyle(currentElement.shadowRoot!, p);
      }
    });

    return (
      <div class="p-4 my-1 border border-red-500 border-dashed text-red-500">
        <p class="font-bold pb-4">
          {`${title()}「`}
          <Name name={props["call-name"]} />
          {`」失败：`}
        </p>
        <ErrorMessage
          errorType={props["error-type"]}
          errorValue={props["error-value"]}
        />
      </div>
    );
  };
}

export function registerCustomElement(tag: string) {
  customElement(
    tag,
    { "call-type": "", "call-name": "", "error-type": "", "error-value": null },
    createBlockCallErrorComponent(),
  );
}
