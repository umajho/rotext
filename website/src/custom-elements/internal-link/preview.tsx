import {
  Component,
  createEffect,
  createMemo,
  createResource,
  on,
  onMount,
} from "solid-js";
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
import { styleProvider as styleProviderForTailwind } from "../../styles/tailwind";

import { closestScrollContainer } from "../../utils/mod";
import Global from "../../global";
import { wikiResourceManager } from "../../resource-managers/wiki";
import { navigateToWiki } from "../../utils/navigation";

export function createDemoPreviewRenderer(
  createRendererOpts: { proseClass: string; proseStyleProvider: StyleProvider },
): AnkorWidgetNavigationInnerRenderer {
  const { proseClass: _, proseStyleProvider } = createRendererOpts;

  return (rawAddrW, rendererOpts) => {
    {
      const authRawAddrCurValue = //
        getAuthenticFullPageNameOrAnchor(rawAddrW.currentValue);
      rendererOpts.updateNavigationText(`[[${authRawAddrCurValue}]]`);
    }

    let el!: HTMLElement;

    return {
      // 虽然可以在缓存已存在时允许自动打开，但为了页面的整洁，决定还是不自动打
      // 开任何内部链接挂件。
      isAutoOpenable: false,
      render: (el_, renderOpts) => {
        el = el_;
        const dispose = render(() => {
          const rawAddr = createSignalGetterFromWatchable(rawAddrW);
          return (
            <Preview
              rawAddress={rawAddr()}
              proseStyleProvider={proseStyleProvider}
              updateNavigationText={rendererOpts.updateNavigationText}
            />
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
        const currentPage = Global.currentPageName!;

        const { page, section } = parseAddress(rawAddrW.currentValue);

        if (page && page !== currentPage) {
          if (section) {
            navigateToWiki(`${page}#${section}`);
          } else {
            navigateToWiki(page);
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

const Preview: Component<{
  rawAddress: string;

  proseStyleProvider: StyleProvider;
  updateNavigationText: (v: string) => void;
}> = (props) => {
  const authRawAddr = createMemo(() =>
    getAuthenticFullPageNameOrAnchor(props.rawAddress)
  );
  createEffect(() => props.updateNavigationText(`[[${authRawAddr()}]]`));
  const address = createMemo(() => parseAddress(props.rawAddress));

  let contentContainerEl!: HTMLDivElement;

  const [pageHTML] = createResource(() => {
    const page = address().page;
    if (!page) return null;
    return wikiResourceManager.getPage(page);
  });
  onMount(() => {
    createEffect(on([pageHTML], ([pageHTML]) => {
      if (!pageHTML) {
        contentContainerEl.innerHTML = "TODO";
      } else {
        contentContainerEl.innerHTML = "";
        contentContainerEl.append(pageHTML.cloneNode(true));
      }
    }));
  });

  const widgetOwnerData = JSON.stringify(
    {
      // FIXME!!!: 目前只是用来占位。正确的值应该根据其外层的挂件来计算。
      level: 1,
    } satisfies Ankor.WidgetOwnerRaw,
  );

  return (
    <ShadowRootAttacher
      styleProviders={[
        styleProviderForPreflight,
        styleProviderForTailwind,
        props.proseStyleProvider,
      ]}
    >
      <div
        class={Ankor.WIDGET_OWNER_CLASS}
        data-ankor-widget-owner={widgetOwnerData}
      >
        <div class={`${Ankor.CONTENT_CLASS} p-2 md:p-4`}>
          <div class={`${Ankor.ANCHOR_CLASS} relative z-10`} />
          <div
            ref={contentContainerEl}
            class="tuan-background tuan-prose break-all"
          />
        </div>
      </div>
    </ShadowRootAttacher>
  );
};

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

function getAuthenticFullPageNameOrAnchor(address: string) {
  if (address.startsWith("#")) return address;
  return wikiResourceManager.getAuthenticFullPageName(address);
}
