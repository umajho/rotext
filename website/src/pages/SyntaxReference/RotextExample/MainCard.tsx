import { Component } from "solid-js";

import { Card } from "../../../components/ui/mod";

import { RotextExampleStore } from "./store";

import { createInputParts } from "./input-parts/mod";
import { createOutputParts } from "./output-parts/mod";

export const MainCard: Component<{
  store: RotextExampleStore;
}> = (props) => {
  const { InputTopBar, InputPane } = createInputParts(props.store);
  const { OutputTopBar, OutputPane } = createOutputParts(props.store);

  return (
    <>
      <div class="relative">
        <div class="absolute z-10 left-4">
          <div class="bg-indigo-800 px-8 py-2 rounded-lg">
            示例
          </div>
        </div>
      </div>
      <div class="px-4 py-6">
        <Card
          class="bg-indigo-800"
          bodyClass="max-sm:px-1 max-sm:py-1 px-4 py-0"
        >
          <div class="grid grid-cols-1 xl:grid-cols-2">
            <div class="max-xl:order-1">
              {InputTopBar}
            </div>
            <div class="max-xl:order-3">
              {OutputTopBar}
            </div>
            <div class="max-xl:order-2">
              {InputPane}
            </div>
            <div class="max-xl:order-4">
              {OutputPane}
            </div>
          </div>
          TODO!!!: empty, group, example-fixture (name), (preview).
        </Card>
      </div>
    </>
  );
};
