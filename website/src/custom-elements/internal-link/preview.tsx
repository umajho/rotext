import {
  Accessor,
  Component,
  createEffect,
  createMemo,
  createSignal,
  on,
  onMount,
  Show,
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
import { Button, Loading } from "../../components/ui/mod";

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

          const [content, { reload: reloadPreviewContent_ }] =
            createPreviewContent(
              addr,
              { parentAddress: widgetOwnerAgent.address },
            );
          function reloadPreviewContent() {
            reloadPreviewContent_(addr());
          }

          return (
            <Preview
              parentLevel={widgetOwnerAgent.level}
              address={addr()}
              content={content()}
              isLoading={!content()}
              reloadPreviewContent={reloadPreviewContent}
              proseStyleProvider={proseStyleProvider}
            />
          );
        }, el);
        renderOpts.onCleanup(dispose);
      },
    };
  };
}

type ParentAddress = ["reference" | "internal", string] | ["special", string];

function navigate(address: string, opts: {
  parentAddress: ParentAddress;
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
  /** 输入变更。 */
  | ["input_changed"]
  | ["todo"];

function createPreviewContent(
  address: Accessor<Address>,
  opts: GetPreviewContentOptions,
): [Accessor<PreviewContent | null>, { reload: (address: Address) => void }] {
  let initialAddress = address();
  let currentAddressInString = stringifyAddress(initialAddress);
  const [content, setContent] = createSignal<PreviewContent | null>(null);

  createEffect(
    on([address], () => setContent(["input_changed"]), { defer: true }),
  );

  async function updatePreviewAddress(
    address: Address,
    opts: GetPreviewContentOptions,
  ) {
    const addressInString = stringifyAddress(address);
    currentAddressInString = addressInString;
    const content = await getPreviewContent(address, opts);
    if (currentAddressInString !== addressInString) return;
    setContent(content);
  }

  // 不 await。
  updatePreviewAddress(initialAddress, opts);

  return [content, {
    reload: (address) => {
      updatePreviewAddress(address, opts);
    },
  }];
}

interface GetPreviewContentOptions {
  parentAddress: ParentAddress;
}

async function getPreviewContent(
  address: Address,
  opts: GetPreviewContentOptions,
): Promise<PreviewContent> {
  let fullPageName = address.page;
  const heading = address.heading;
  if (!fullPageName) {
    switch (opts.parentAddress[0]) {
      case "reference":
        return ["todo"];
      case "internal":
        fullPageName = opts.parentAddress[1];
        break;
      case "special": {
        if (opts.parentAddress[1] === "live") return ["live"];
        return ["not_capable"];
      }
      default:
        opts.parentAddress satisfies never;
    }
  }
  const page = await wikiResourceManager.getPage(fullPageName!);
  if (!page) return ["page_not_found"];
  const cutContent = cutAndCloneContentForPreview(page, heading);
  if (!cutContent) return ["heading_not_found"];
  return ["ok", cutContent];
}

const HINT_CLASS = "flex justify-center h-full text-gray-400 font-black";

const Preview: Component<{
  parentLevel: number;
  address: Address;
  content: PreviewContent | null;
  isLoading: boolean;

  reloadPreviewContent: () => void;

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

      let hint: string | undefined;
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
        case "input_changed":
          break; // 此情况的 hint 不放在 `contentContainerEl` 里。
        default:
          content satisfies never;
      }

      hint && contentContainerEl.append(createHintElement(hint));
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
      <Show when={props.isLoading}>
        <div class="flex w-full justify-center items-center p-8">
          <Loading />
        </div>
      </Show>
      <div
        class={[
          Ankor.WIDGET_OWNER_CLASS,
          ...(props.isLoading ? ["hidden"] : []),
        ].join(" ")}
        data-ankor-widget-owner={widgetOwnerData()}
      >
        <div class={`${Ankor.CONTENT_CLASS} p-2 md:p-4`}>
          <div class={`${Ankor.ANCHOR_CLASS} relative z-10`} />
          <Show when={props.content?.[0] === "input_changed"}>
            <div class={HINT_CLASS}>
              输入变更，
              <Button
                class="inline-flex"
                type="primary"
                size="xs"
                onClick={props.reloadPreviewContent}
              >
                刷新
              </Button>
              。
            </div>
          </Show>
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
function stringifyAddress(address: Address): string {
  if (!address.heading) return address.page ?? "";
  return `${address.page ?? ""}#${address.heading}`;
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
  return createDivElement({ textContent: hint!, class: HINT_CLASS });
}

function createDivElement(opts: { textContent: string; class: string }) {
  const el = document.createElement("div");
  el.textContent = opts.textContent;
  el.className = opts.class;
  return el;
}
