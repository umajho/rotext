import { Component } from "solid-js";
import { customElement, noShadowDOM } from "solid-element";

import { ShadowRootAttacher } from "@rolludejo/web-internal";

import { styleProvider as styleProviderForPreflight } from "../../../styles/preflight";
import { styleProvider as styleProviderForTailwind } from "../../../styles/tailwind";
import { styleProvider as styleProviderForOneDark } from "../../../styles/one-dark";
import { styleProvider as styleProviderForTuanProse } from "../../../styles/tuan-prose";

import { createRotextExampleStore } from "./store";

import { MainCard } from "./MainCard";

export function registerCustomElement(tag: string, opts: {
  getFixtures: (fixtureNames: Set<string>) => { [fixtureName: string]: string };
}) {
  customElement(
    tag,
    { input: "", expected: null, "use-fixtures": "" },
    createRotextExampleComponent(opts),
  );
}

function createRotextExampleComponent(opts: {
  getFixtures: (fixtureNames: Set<string>) => { [fixtureName: string]: string };
}): Component<
  { input: string; expected: string | null; "use-fixtures": string }
> {
  return (props) => {
    noShadowDOM();

    const fixtureNames = props["use-fixtures"]
      ? props["use-fixtures"].split(",")
      : null;
    const fixtures = fixtureNames
      ? opts.getFixtures(new Set(fixtureNames))
      : null;

    const store = createRotextExampleStore({
      originalInput: props.input,
      originalExpected: props.expected ?? "",
      fixtureNames,
      fixtures,
    });

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
