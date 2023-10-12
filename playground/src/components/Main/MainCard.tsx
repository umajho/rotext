import { Component } from "solid-js";

import * as examples from "@rotext/example-documentations";

import { SUPPORTS_DVH } from "@rotext/web-utils";

import { Card } from "../ui";

import { createEditorStore } from "../../hooks/editor-store";

import { createPreviewParts } from "./preview-parts/mod";
import { createEditorParts } from "./editor-parts/mod";

const SIZE_OPTS = {
  widthClass: "w-full sm:w-[80vw] lg:max-w-[35rem] lg:w-[45vw]",
  heightClass: SUPPORTS_DVH
    ? `h-[calc(50dvh-6rem)] sm:h-[calc(50dvh-8rem)] lg:h-[calc(100dvh-16rem)]`
    : `h-[calc(50vh-6rem)] sm:h-[calc(50vh-8rem)] lg:h-[calc(100vh-16rem)]`,
};

const MainCard: Component = () => {
  const store = createEditorStore(examples.introduction);

  const { EditorTopBar, Editor } = createEditorParts(store, SIZE_OPTS);
  const { PreviewTopBar, Preview } = createPreviewParts(store, SIZE_OPTS);

  return (
    <Card class="h-content" bodyClass="max-sm:px-1 max-sm:py-1">
      <div class="grid grid-cols-1 lg:grid-cols-2">
        <div class="max-lg:order-1">
          {EditorTopBar}
        </div>
        <div class="max-lg:order-3">
          {PreviewTopBar}
        </div>
        <div class="max-lg:order-2">
          {Editor}
        </div>
        <div class="max-lg:order-4">
          {Preview}
        </div>
      </div>
    </Card>
  );
};
export default MainCard;
