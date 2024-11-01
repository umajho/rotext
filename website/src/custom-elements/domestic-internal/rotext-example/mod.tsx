import { Component } from "solid-js";
import { customElement, getCurrentElement, noShadowDOM } from "solid-element";

import * as Ankor from "ankor";

import { findClosestElementEx } from "@rolludejo/internal-web-shared/dom";
import { ShadowRootAttacher } from "@rolludejo/internal-web-shared/shadow-root";

import { styleProvider as styleProviderForPreflight } from "../../../styles/preflight";
import { styleProvider as styleProviderForTailwind } from "../../../styles/tailwind";
import { styleProvider as styleProviderForOneDark } from "../../../styles/one-dark";
import { styleProvider as styleProviderForTuanProse } from "../../../styles/tuan-prose";

import { createRotextExampleStore } from "./create-store";

import { MainCard } from "./MainCard";

export function registerCustomElement(tag: string) {
  customElement(
    tag,
    { input: "", expected: null, "use-fixtures": "" },
    createRotextExampleComponent(),
  );
}

function createRotextExampleComponent(): Component<
  { input: string; expected: string | null; "use-fixtures": string }
> {
  return (props) => {
    noShadowDOM();

    const fixtureNames = props["use-fixtures"]
      ? props["use-fixtures"].split(",")
      : null;
    const fixtures = fixtureNames
      ? getFixtures(
        findClosestElementEx(getCurrentElement(), (el) =>
          el.classList.contains(Ankor.CONTENT_CLASS))!,
        new Set(fixtureNames),
      )
      : null;

    const store = createRotextExampleStore({
      originalInput: props.input,
      expectedOutputHTMLForOriginalInput: props.expected ?? "",
      fixtureNames,
      fixtures,
    });

    getCurrentElement().verifyOutputOfOriginalInput = (
      report: (matches: boolean) => void,
    ) => {
      store.onOutputOfOriginalInputVerified(report);
      store.verifyOutputOfOriginalInput();
    };

    return (
      <ShadowRootAttacher
        styleProviders={[
          styleProviderForPreflight,
          styleProviderForTailwind,
          styleProviderForOneDark,
          styleProviderForTuanProse,
        ]}
      >
        <MainCard store={store} />
      </ShadowRootAttacher>
    );
  };
}

function getFixtures(
  contentContainerEl: HTMLElement,
  fixtureNames: Set<string>,
): { [fixtureName: string]: string } {
  const els = contentContainerEl.querySelectorAll("x-rotext-example-fixture");
  const qualifiedEls = [...els]
    .filter((el) => fixtureNames.has(el.getAttribute("name")!));

  return Object.fromEntries(
    qualifiedEls.map((
      el,
    ) => [el.getAttribute("name")!, el.getAttribute("input")!]),
  );
}
