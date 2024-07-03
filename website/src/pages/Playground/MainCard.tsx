import { Component } from "solid-js";

import { SUPPORTS_DVH } from "@rotext/web-utils";

import { Card } from "../../components/ui/mod";

import { createEditorStore } from "../../hooks/editor-store";

import { createPreviewParts } from "./preview-parts/mod";
import { createEditorParts } from "./editor-parts/mod";

const SIZE_OPTS = {
  widthClass: "w-full",
  heightClass: SUPPORTS_DVH
    ? `h-[calc(50dvh-6rem)] sm:h-[calc(50dvh-8rem)] xl:h-[calc(100dvh-16rem)]`
    : `h-[calc(50vh-6rem)] sm:h-[calc(50vh-8rem)] xl:h-[calc(100vh-16rem)]`,
};

const DOC_INTRODUCTION = await (await fetch("static/docs/rotext入门.rotext"))
  .text();

const MainCard: Component = () => {
  const store = createEditorStore(DOC_INTRODUCTION);

  const { EditorTopBar, Editor } = createEditorParts(store, SIZE_OPTS);
  const { PreviewTopBar, Preview } = createPreviewParts(store, SIZE_OPTS);

  return (
    <Card
      class="h-content xl:max-w-[80rem]"
      bodyClass="max-sm:px-1 max-sm:py-1"
    >
      <div class="grid grid-cols-1 xl:grid-cols-2">
        <div class="max-xl:order-1">
          {EditorTopBar}
        </div>
        <div class="max-xl:order-3">
          {PreviewTopBar}
        </div>
        <div class="max-xl:order-2">
          {Editor}
        </div>
        <div class="max-xl:order-4">
          {Preview}
        </div>
      </div>
    </Card>
  );
};
export default MainCard;
