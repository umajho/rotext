import { createEffect } from "solid-js";
import { render } from "solid-js/web";

import {
  ShadowRootAttacher,
  StyleProvider,
} from "@rolludejo/internal-web-shared/shadow-root";

import { AnkorWidgetNavigationInnerRenderer } from "@rotext/solid-components/internal";

import { createSignalGetterFromWatchable } from "../hooks";

import { styleProvider as styleProviderForPreflight } from "../../styles/preflight";

export function createDemoPreviewRenderer(
  createRendererOpts: { proseClass: string; proseStyleProvider: StyleProvider },
): AnkorWidgetNavigationInnerRenderer {
  const { proseClass: _, proseStyleProvider } = createRendererOpts;

  return (rawAddrW, rendererOpts) => {
    rendererOpts.updateNavigationText(`[[${rawAddrW.currentValue}]]`);

    return {
      isAutoOpenable: false,
      render: (el, renderOpts) => {
        const dispose = render(() => {
          const address = createSignalGetterFromWatchable(rawAddrW);
          createEffect(() =>
            rendererOpts.updateNavigationText(`[[${address()}]]`)
          );
          return (
            <ShadowRootAttacher
              styleProviders={[styleProviderForPreflight, proseStyleProvider]}
            >
              <span>{`TODO: ${address()}`}</span>
            </ShadowRootAttacher>
          );
        }, el);
        renderOpts.onCleanup(dispose);
      },
      navigate: () => {
        window.alert(`TODO: 前往 [[${rawAddrW.currentValue}]]。`);
      },
    };
  };
}
