import { Component } from "solid-js";

import { SUPPORTS_DVH } from "@rotext/web-utils";

import { Card } from "../../components/ui/mod";

import { createEditorStore } from "./editor-store";

import { PreviewTopBar, PreviewWrapper } from "./preview-parts/mod";
import { EditorTopBar, EditorWrapper } from "./editor-parts/mod";
import { createEditorPartStore } from "./editor-parts/store";
import { createPreviewPartStore } from "./preview-parts/store";

import examples from "./examples";

const SIZE_OPTS = {
  widthClass: "w-full",
  heightClass: SUPPORTS_DVH
    ? `h-[calc(50dvh-6rem)] sm:h-[calc(50dvh-8rem)] xl:h-[calc(100dvh-16rem)]`
    : `h-[calc(50vh-6rem)] sm:h-[calc(50vh-8rem)] xl:h-[calc(100vh-16rem)]`,
};

const MainCard: Component = () => {
  const editorStore = createEditorStore(examples.get("入门"));
  const editorPartStore = createEditorPartStore();
  const previewPartStore = createPreviewPartStore();

  return (
    <Card
      class="h-content w-full xl:w-[80rem]"
      bodyClass="max-sm:px-1 max-sm:py-1"
    >
      <div class="grid grid-cols-1 xl:grid-cols-2">
        <div class="max-xl:order-1">
          <EditorTopBar store={editorPartStore} editorStore={editorStore} />
        </div>
        <div class="max-xl:order-3">
          <PreviewTopBar store={previewPartStore} />
        </div>
        <div class="max-xl:order-2">
          <EditorWrapper
            store={editorPartStore}
            editorStore={editorStore}
            {...SIZE_OPTS}
          />
        </div>
        <div class="max-xl:order-4">
          <PreviewWrapper
            store={previewPartStore}
            editorStore={editorStore}
            {...SIZE_OPTS}
          />
        </div>
      </div>
    </Card>
  );
};
export default MainCard;
