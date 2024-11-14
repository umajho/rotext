import {
  Component,
  createEffect,
  createMemo,
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

import { createSignalGetterFromWatchable } from "../../hooks";

import { styleProvider as styleProviderForPreflight } from "../../../styles/preflight";
import { styleProvider as styleProviderForTailwind } from "../../../styles/tailwind";

import { closestScrollContainer } from "../../../utils/mod";
import { navigateToAddress } from "../../../utils/navigation";
import { Button, Loading } from "../../../components/ui/mod";
import { createPreviewContent, PreviewContent } from "../shared/mod";
import { Address, reconstructAddressAsText } from "../../../utils/address";

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
      render: (elIn, renderOpts) => {
        widgetOwnerAgent = renderOpts.widgetOwnerAgent;
        el = elIn;
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

          const addr = createMemo(() => {
            const local = parseLocalAddress(rawAddr());
            const parent = getParentAddress(widgetOwnerAgent.element);
            const authentic = ((): Address => {
              switch (local[0]) {
                case "anchor":
                  if (parent[0] === "internal") {
                    const authentic: Address = [...parent];
                    authentic[2] = local[1];
                    return authentic;
                  } else {
                    return parent;
                  }
                case "internal":
                  return local;
              }
            })();

            return { local, authentic };
          });

          createEffect(on([addr], ([addr]) => {
            const text = (() => {
              if (addr.local[0] === "anchor") {
                return `[[#${addr.local[1]}]]`;
              } else {
                // FIXME: 去掉 any。
                return reconstructAddressAsText(addr.authentic as any);
              }
            })();

            rendererOpts.updateNavigation({
              text,
              action: () =>
                navigate(addr, {
                  isParentOutmost: widgetOwnerAgent.level === 1,
                  tryScrollToHeading,
                  scrollToParentContentTop,
                }),
            });
          }));

          const [content, contentOpts] = createPreviewContent(() =>
            addr().authentic
          );
          function reloadPreviewContent() {
            contentOpts.reload(addr().authentic);
          }

          return (
            <Preview
              parentLevel={widgetOwnerAgent.level}
              address={addr().authentic}
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

function getParentAddress(widgetOwnerEl: HTMLElement): Address {
  return JSON.parse(widgetOwnerEl.dataset.address!);
}

function navigate(address: { local: LocalAddress; authentic: Address }, opts: {
  isParentOutmost: boolean;

  tryScrollToHeading?: (heading: string) => boolean;
  scrollToParentContentTop?: () => void;
}) {
  if (address.local[0] === "anchor") { // 地址只有 hash 部分时才会进入此分支。
    const heading = address.local[1];
    if (heading) {
      if (opts.tryScrollToHeading?.(heading)) return;
      navigateToAddress(address.authentic, {
        shouldOpenNewTab: !opts.isParentOutmost,
      });
    } else {
      opts.scrollToParentContentTop?.();
    }
  } else {
    navigateToAddress(address.authentic);
  }
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
    JSON.stringify(
      { level: props.parentLevel + 1 } satisfies Ankor.WidgetOwnerRaw,
    )
  );
  const addressData = createMemo(() =>
    JSON.stringify(props.address satisfies Address)
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
        data-address={addressData()}
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

type LocalAddress =
  | Extract<Address, { 0: "internal" }>
  | ["anchor", string | null];

function parseLocalAddress(raw: string): LocalAddress {
  const [page, anchor] = raw.split("#", 2);

  if (page) {
    return ["internal", page, anchor || null];
  } else {
    return ["anchor", anchor ?? null];
  }
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

function createHintElement(hint: string) {
  return createDivElement({ textContent: hint!, class: HINT_CLASS });
}

function createDivElement(opts: { textContent: string; class: string }) {
  const el = document.createElement("div");
  el.textContent = opts.textContent;
  el.className = opts.class;
  return el;
}
