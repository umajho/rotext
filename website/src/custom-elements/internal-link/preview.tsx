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
import { wikiResourceManager } from "../../resource-managers/wiki";
import { navigateToAddress, navigateToWiki } from "../../utils/navigation";

export function createDemoPreviewRenderer(
  createRendererOpts: { proseClass: string; proseStyleProvider: StyleProvider },
): AnkorWidgetNavigationInnerRenderer {
  const { proseClass: _, proseStyleProvider } = createRendererOpts;

  return (rawAddrW, rendererOpts) => {
    let widgetOwnerAgent!: Ankor.WidgetOwnerAgent;
    let el!: HTMLElement;

    return {
      // 虽然可以在缓存已存在时允许自动打开，但为了页面的整洁，决定还是不自动打
      // 开任何内部链接挂件。
      isAutoOpenable: false,
      render: (el_, renderOpts) => {
        widgetOwnerAgent = renderOpts.widgetOwnerAgent;
        el = el_;
        const dispose = render(() => {
          const rawAddr = createSignalGetterFromWatchable(rawAddrW);

          function tryScrollToHeading(heading: string): boolean {
            const els = getContentElementAndScrollContainerElement(el);
            if (els) {
              const hEl = [...els.content.querySelectorAll("h1,h2,h3,h4,h5,h6")]
                .find((hEl) => hEl.textContent === heading);
              if (hEl) {
                els.scrollContainer.scrollTop =
                  hEl.getBoundingClientRect().top -
                  els.content.getBoundingClientRect().top;
                return true;
              }
            }
            return false;
          }

          function scrollToParentContentTop() {
            const els = getContentElementAndScrollContainerElement(el);
            if (els) {
              els.scrollContainer.scrollTop = 0;
            }
          }

          createEffect(on([rawAddr], ([rawAddr]) => {
            const authRawAddr = getAuthenticFullPageNameOrAnchor(rawAddr);
            rendererOpts.updateNavigation({
              text: authRawAddr ?? rawAddr,
              action: authRawAddr
                ? (() =>
                  navigate(authRawAddr, {
                    parentAddress: widgetOwnerAgent.address,
                    isParentOutmost: widgetOwnerAgent.level === 1,
                    tryScrollToHeading,
                    scrollToParentContentTop,
                  }))
                : null,
            });
          }));

          const addr = createMemo(() => parseAddress(rawAddr()));

          const [content] = createResource(
            async (): Promise<PreviewContent> => {
              let fullPageName = addr().page;
              const heading = addr().heading;
              if (!fullPageName) {
                switch (widgetOwnerAgent.address[0]) {
                  case "reference":
                    return ["todo"];
                  case "internal":
                    fullPageName = widgetOwnerAgent.address[1];
                    break;
                  case "special": {
                    if (widgetOwnerAgent.address[1] === "live") return ["live"];
                    return ["not_capable"];
                  }
                  default:
                    widgetOwnerAgent.address satisfies never;
                }
              }
              const page = await wikiResourceManager.getPage(fullPageName!);
              if (!page) return ["page_not_found"];
              const cutContent = cutAndCloneContentForPreview(page, heading);
              if (!cutContent) return ["heading_not_found"];
              return ["ok", cutContent];
            },
          );

          return (
            <Preview
              parentLevel={widgetOwnerAgent.level}
              address={addr()}
              content={content()}
              proseStyleProvider={proseStyleProvider}
            />
          );
        }, el);
        renderOpts.onCleanup(dispose);
      },
    };
  };
}

function navigate(address: string, opts: {
  parentAddress: ["reference" | "internal", string] | ["special", string];
  isParentOutmost: boolean;

  tryScrollToHeading?: (heading: string) => boolean;
  scrollToParentContentTop?: () => void;
}) {
  const { page, heading } = parseAddress(address);

  if (page) {
    if (heading) {
      navigateToWiki(`${page}#${heading}`);
    } else {
      navigateToWiki(page);
    }
  } else if (heading) { // 地址只有 hash 部分。
    if (opts.tryScrollToHeading?.(heading)) return;
    if (opts.parentAddress[0] === "special") return; // 忽略无法导航的特殊情况。
    navigateToAddress(opts.parentAddress, {
      heading,
      shouldOpenNewTab: !opts.isParentOutmost,
    });
  } else {
    opts.scrollToParentContentTop?.();
  }
}

type PreviewContent =
  | ["ok", CutAndCloneContentForPreviewResult]
  /** 没有找到对应页面。 */
  | ["page_not_found"]
  /** 找到了对应页面，但没有对应标题。 */
  | ["heading_not_found"]
  /** 是正在进行编辑的内容，无法提供预览。 */
  | ["live"]
  /** 由于其他原因无法提供预览。 */
  | ["not_capable"]
  | ["todo"];

const Preview: Component<{
  parentLevel: number;
  address: Address;
  content?: PreviewContent;

  proseStyleProvider: StyleProvider;
}> = (props) => {
  let contentContainerEl!: HTMLDivElement;

  onMount(() => {
    createEffect(on([() => props.content], ([content]) => {
      contentContainerEl.innerHTML = "";
      if (!content) return;

      if (content[0] === "ok") {
        const cutResult = content[1];

        if (cutResult.hasContentBefore) {
          contentContainerEl.append(createHintElement("…"));
        }
        // `content[1]` 是专门为了预览而克隆的，因此不用再克隆一遍。
        contentContainerEl.append(...cutResult.content);
        if (cutResult.hasContentAfter) {
          contentContainerEl.append(createHintElement("…"));
        }
        return;
      }

      let hint: string;
      switch (content[0]) {
        case "page_not_found":
          hint = "（页面不存在）";
          break;
        case "heading_not_found":
          hint = "（页面中不存在该标题）";
          break;
        case "live":
          hint = "（不支持预览实时改变的内容）";
          break;
        case "not_capable":
          hint = "（无法预览）";
          break;
        case "todo":
          hint = "（TODO）";
          break;
        default:
          content satisfies never;
      }

      contentContainerEl.append(createHintElement(hint!));
    }));
  });

  const widgetOwnerData = createMemo(() =>
    JSON.stringify(((): Ankor.WidgetOwnerRaw => {
      const page = props.address.page;
      return {
        level: props.parentLevel + 1,
        address: page ? ["internal", page] : ["special", "never"],
      };
    })())
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
        data-ankor-widget-owner={widgetOwnerData()}
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
  heading: string | null;
}

function parseAddress(raw: string): Address {
  const [page, heading] = raw.split("#", 2);
  return {
    page: page || null,
    heading: heading ?? null,
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

interface CutAndCloneContentForPreviewResult {
  content: NodeListOf<ChildNode>;
  hasContentBefore: boolean;
  hasContentAfter: boolean;
}

function cutAndCloneContentForPreview(
  fullContent: DocumentFragment,
  heading: string | null,
): CutAndCloneContentForPreviewResult | null {
  let startNode: Node | null;
  if (heading) {
    startNode = [...fullContent.querySelectorAll("h1,h2,h3,h4,h5,h6")]
      .find((hEl) => hEl.textContent === heading) ?? null;
  } else {
    startNode = fullContent.firstElementChild;
  }
  if (!startNode) return null;
  let curNode: Node | null = startNode;

  const hasContentBefore = curNode.parentNode!.firstChild !== curNode;

  // 由于不明原因，使用 `template` 元素，并通过其 `.content` 获取到的
  // DocumentFragment 总是为空（Chrome、Safari 都是如此）。
  // 额外观察：在 Chrome Dev 控制台中将 log 出来的 `template` 元素存为临时变量，
  // 打印时显示其中有预期的内容，但无论 `.content` 还是 `.innerHTML` 中都没有内
  // 容，直接打 log 也是如此。
  // 退而使用 `div` 元素。
  const tEl = document.createElement("div");

  let shouldStopBeforeHeading = false;
  while (curNode) {
    if ("tagName" in curNode && /^H[1-6]$/.test((curNode as Element).tagName)) {
      if (shouldStopBeforeHeading) break;
    } else {
      shouldStopBeforeHeading = true;
    }
    tEl.append(curNode.cloneNode(true));
    curNode = curNode.nextSibling;
  }

  if (tEl.childElementCount || !heading) {
    const hasContentAfter = !!curNode;

    return {
      content: tEl.childNodes,
      hasContentBefore,
      hasContentAfter,
    };
  } else {
    return null;
  }
}

function createHintElement(hint: string) {
  return createDivElement({
    textContent: hint!,
    class: "flex justify-center h-full text-gray-400 font-black",
  });
}

function createDivElement(opts: { textContent: string; class: string }) {
  const el = document.createElement("div");
  el.textContent = opts.textContent;
  el.className = opts.class;
  return el;
}
