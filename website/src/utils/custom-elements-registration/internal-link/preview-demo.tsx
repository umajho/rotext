import { render } from "solid-js/web";

import {
  ShadowRootAttacher,
  StyleProvider,
} from "@rolludejo/web-internal/shadow-root";

import { AnkorWidgetNavigationInnerRenderer } from "@rotext/solid-components/internal";

import { styleProvider as styleProviderForPreflight } from "../../../styles/preflight";

export function createDemoPreviewRenderer(
  opts: { proseClass: string; proseStyleProvider: StyleProvider },
): AnkorWidgetNavigationInnerRenderer {
  return (el, rawAddr, rOpts) => {
    const dispose = render(() => {
      return (
        <ShadowRootAttacher
          styleProviders={[styleProviderForPreflight, opts.proseStyleProvider]}
        >
          <span>{`TODO: ${rawAddr}`}</span>
        </ShadowRootAttacher>
      );
    }, el);
    rOpts.onCleanup(dispose);
  };
}
