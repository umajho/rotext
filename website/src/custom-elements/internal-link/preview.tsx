import { createEffect, createMemo } from "solid-js";
import { render } from "solid-js/web";

import * as Ankor from "ankor";

import { findClosestElementEx } from "@rolludejo/internal-web-shared/dom";
import {
  ShadowRootAttacher,
  StyleProvider,
} from "@rolludejo/internal-web-shared/shadow-root";

import { AnkorWidgetNavigationInnerRenderer } from "@rotext/solid-components/internal";

import { createSignalGetterFromWatchable } from "../hooks";

import { styleProvider as styleProviderForPreflight } from "../../styles/preflight";
import { closestScrollContainer } from "../../utils/mod";
import Global from "../../global";

export function createDemoPreviewRenderer(
  createRendererOpts: { proseClass: string; proseStyleProvider: StyleProvider },
): AnkorWidgetNavigationInnerRenderer {
  const { proseClass: _, proseStyleProvider } = createRendererOpts;

  return (rawAddrW, rendererOpts) => {
    rendererOpts.updateNavigationText(`[[${rawAddrW.currentValue}]]`);

    let el!: HTMLElement;

    return {
      // 虽然可以在缓存已存在时允许自动打开，但为了页面的整洁，决定还是不自动打
      // 开任何内部链接挂件。
      isAutoOpenable: false,
      render: (el_, renderOpts) => {
        el = el_;
        const dispose = render(() => {
          const rawAddr = createSignalGetterFromWatchable(rawAddrW);
          createEffect(() =>
            rendererOpts.updateNavigationText(`[[${rawAddr()}]]`)
          );
          const address = createMemo(() => parseAddress(rawAddr()));

          return (
            <ShadowRootAttacher
              styleProviders={[styleProviderForPreflight, proseStyleProvider]}
            >
              <span>{`TODO: ${rawAddr()}`}</span>
            </ShadowRootAttacher>
          );
        }, el);
        renderOpts.onCleanup(dispose);
      },
      navigate: () => {
        // TODO!!: 更好的方式。比如在 widget owner 的信息中嵌入地址（页面名）。不
        // 过要记得比对的应该是最外层 widget owner 嵌入的地址（页面名），因为导航
        // 是以最外层为基准。另外，地址不一定非要放入 `data-ankor-widget-owner`
        // 当中，也可以放在 `data-address`、`data-page-name`、`data-post-id` 之
        // 类其他属性里。
        // 对于导航类挂件（如本挂件及引用链接）：在最外层有多个 widget owner 时
        // （比如在一个串中），应当额外考虑目标地址是否已经处于页面当中（已经加载）。
        const currentPage = window.location.pathname.split("/").at(-1);

        const { page, section } = parseAddress(rawAddrW.currentValue);

        if (page && page !== currentPage) {
          if (section) {
            Global.navigator!(`/syntax-reference/${page}#${section}`);
          } else {
            Global.navigator!(`/syntax-reference/${page}`);
          }
        } else if (section) {
          const els = getContentElementAndScrollContainerElement(el);
          if (els) {
            for (const h of els.content.querySelectorAll("h1,h2,h3,h4,h5,h6")) {
              if (h.textContent === section) {
                els.scrollContainer.scrollTop = h.getBoundingClientRect().top -
                  els.content.getBoundingClientRect().top;
              }
            }
            // 当不存在匹配的标题时，action 会被禁用，因而根本不会调用到此函数，
            // 所以不用处理这种情况。
          }
        } else {
          const els = getContentElementAndScrollContainerElement(el);
          if (els) {
            els.scrollContainer.scrollTop = 0;
          }
        }
      },
    };
  };
}

interface Address {
  page: string | null;
  section: string | null;
}

function parseAddress(raw: string): Address {
  const [page, section] = raw.split("#", 2);
  return {
    page: page || null,
    section: section ?? null,
  };
}

function getContentElementAndScrollContainerElement(el: HTMLElement) {
  const contentEl = closestWidgetOwnerElement(el)
    ?.getElementsByClassName(Ankor.CONTENT_CLASS)[0] as HTMLElement;
  if (contentEl) {
    const scrollContainerEl = closestScrollContainer(contentEl);
    if (scrollContainerEl) {
      return { content: contentEl, scrollContainer: scrollContainerEl };
    }
  }
  return null;
}

function closestWidgetOwnerElement(el: HTMLElement) {
  return findClosestElementEx(
    el,
    (el) => el.classList.contains(Ankor.WIDGET_OWNER_CLASS),
  );
}
