import {
  Component,
  createEffect,
  createMemo,
  createResource,
  createSignal,
  on,
  Show,
} from "solid-js";
import { useLocation, useNavigate, useParams } from "@solidjs/router";

import { HiSolidCodeBracket } from "solid-icons/hi";

import * as Ankor from "ankor";

import { Loading, Menu, MenuItem } from "../../components/ui/mod";

import "../../styles/tuan-prose";
import { wikiResourceManager } from "../../resource-managers/wiki";
import { initializeGlobal } from "../../global";
import { navigateToAddress } from "../../utils/navigation";
import { Address } from "../../utils/address";

export default (() => {
  let contentContainerEl!: HTMLDivElement;

  const params = useParams();
  const pageName = createMemo(() => decodeURIComponent(params.pageName!));
  const navigate = useNavigate();
  const location = useLocation();

  initializeGlobal({ navigator: navigate });
  createEffect(on([pageName], ([pageName]) => {
    if (pageName !== wikiResourceManager.getAuthenticFullPageName(pageName)) {
      navigateToAddress(["internal", pageName, location.hash || null]);
    }
  }));

  const sourceLink = createMemo(() => {
    const pagePath = pageName().replace(":", "/");
    return `https://github.com/umajho/rotext/blob/main/docs/wiki/${pagePath}.md?plain=1`;
  });

  const [isIndexLoaded, setIsIndexLoaded] = createSignal(false);

  const [pageHTML] = createResource(
    pageName,
    async (pageName) => {
      const page = await wikiResourceManager.getPage(pageName);
      if (!page) {
        navigate("/404");
        return;
      }

      return page;
    },
  );

  createEffect(
    on([pageHTML], ([pageHTML]) => {
      contentContainerEl.parentElement!.scrollTop = 0;
      contentContainerEl.innerHTML = "";
      if (pageHTML) {
        contentContainerEl.append(pageHTML.cloneNode(true));
      }

      if (!isIndexLoaded()) {
        setIsIndexLoaded(true);
      }
    }),
  );
  createEffect(on([() => pageHTML.loading], ([loading]) => {
    if (loading) contentContainerEl.innerHTML = "";
  }));
  createEffect(on([() => location.hash], ([hash]) => {
    if (hash) {
      const targetID = decodeURIComponent(hash).slice(1);
      const target = document.getElementById(targetID);
      target?.scrollIntoView();
    } else {
      contentContainerEl.parentElement!.scrollTop = 0;
    }
  }));

  const isLoading = () => pageHTML.loading || !isIndexLoaded();

  const [verificationStatistics, setVerificationStatistics] = createSignal<
    { total: number; matches: number; mismatches: number } | null
  >(null);
  const verificationStatisticsUnverified = createMemo(() => {
    const statistics = verificationStatistics();
    if (!statistics) return null;
    const unverified = statistics.total - statistics.matches -
      statistics.mismatches;
    return unverified || null;
  });
  createEffect(on([pageHTML], () => setVerificationStatistics(null)));
  function verifyAllOutputsOfOriginalInputs() {
    const rotextExampleEls = [
      ...contentContainerEl.querySelectorAll("x-rotext-example"),
    ];

    setVerificationStatistics({
      total: rotextExampleEls.length,
      matches: 0,
      mismatches: 0,
    });

    for (const el of rotextExampleEls) {
      (el as any).verifyOutputOfOriginalInput((matches: boolean) => {
        const newStatistics = { ...verificationStatistics()! };
        if (matches) {
          newStatistics.matches++;
        } else {
          newStatistics.mismatches++;
        }
        setVerificationStatistics(newStatistics);
      });
    }
  }

  const widgetOwnerData = createMemo(() =>
    JSON.stringify(
      { level: 1 } satisfies Ankor.WidgetOwnerRaw,
    )
  );
  const addressData = createMemo(() =>
    JSON.stringify(["internal", pageName(), null] satisfies Address)
  );

  return (
    <div class="flex flex-col h-full justify-start sm:px-4 lg:px-6 2xl:px-8 gap-2">
      <Show when={isLoading()}>
        <div class="flex w-full h-screen justify-center items-center">
          <Loading />
        </div>
      </Show>
      <div class={`contents h-full w-full ${isLoading() ? "hidden" : ""}`}>
        <div class="flex justify-between content-center w-full">
          <Menu horizontal={true} size="xs" class="bg-base-100">
            <MenuItem>
              <a
                class="tooltip tooltip-bottom"
                href={sourceLink()}
                data-tip="前往源代码"
              >
                <HiSolidCodeBracket size={18} />
              </a>
            </MenuItem>
          </Menu>
          <Menu horizontal={true} size="xs" class="bg-base-100">
            <Show
              when={verificationStatistics()}
              fallback={
                <MenuItem>
                  <a onClick={verifyAllOutputsOfOriginalInputs}>
                    验证本页全部示例输出
                  </a>
                </MenuItem>
              }
            >
              {(statistics) => (
                <span class="flex items-center">
                  本页示例输出验证结果：匹配{" "}
                  <span class="text-green-500">{statistics().matches}
                  </span>，不匹配{" "}
                  <span class="text-red-500">
                    {statistics().mismatches}
                  </span>
                  <Show when={verificationStatisticsUnverified()}>
                    {(unverified) => (
                      <>
                        ，尚未验证{" "}
                        <span class="text-gray-500">{unverified()}</span>
                      </>
                    )}
                  </Show>
                </span>
              )}
            </Show>
          </Menu>
        </div>
        <div
          class={`flex-1 ${Ankor.WIDGET_OWNER_CLASS} overflow-y-scroll overflow-x-hidden`}
          data-ankor-widget-owner={widgetOwnerData()}
          data-address={addressData()}
        >
          <div class="p-4 tuan-background tuan-prose break-all">
            <div class={`${Ankor.ANCHOR_CLASS} relative z-10`} />
            <div class={Ankor.CONTENT_CLASS} ref={contentContainerEl} />
          </div>
        </div>
      </div>
    </div>
  );
}) satisfies Component;
