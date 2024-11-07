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

import * as Ankor from "ankor";

import { Button, Card, Loading } from "../../components/ui/mod";

import "../../styles/tuan-prose";
import { wikiResourceManager } from "../../resource-managers/wiki";
import { initializeGlobal } from "../../global";
import { navigateToWiki } from "../../utils/navigation";

export default (() => {
  let contentContainerEl!: HTMLDivElement;

  const params = useParams();
  const pageName = createMemo(() => decodeURIComponent(params.pageName!));
  const navigate = useNavigate();
  const location = useLocation();

  initializeGlobal({ navigator: navigate });
  createEffect(on([pageName], ([pageName]) => {
    if (pageName !== wikiResourceManager.getAuthenticFullPageName(pageName)) {
      navigateToWiki(pageName);
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
      contentContainerEl.innerHTML = "";
      if (pageHTML) {
        contentContainerEl.append(pageHTML.cloneNode(true));
      }

      if (!isIndexLoaded()) {
        setIsIndexLoaded(true);
      }

      if (location.hash) {
        const targetID = decodeURIComponent(location.hash).slice(1);
        const target = document.getElementById(targetID);
        target?.scrollIntoView();
      } else {
        contentContainerEl.parentElement!.scrollTop = 0;
      }
    }),
  );
  createEffect(on([() => pageHTML.loading], ([loading]) => {
    if (loading) contentContainerEl.innerHTML = "";
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

  const widgetOwnerData = JSON.stringify(
    {
      level: 1,
      address: ["internal", pageName()],
    } satisfies Ankor.WidgetOwnerRaw,
  );

  return (
    <div class="flex max-h-full h-fit justify-center sm:p-4 lg:p-6 2xl:p-8">
      <Card
        class="w-full max-sm:rounded-b-none"
        bodyClass="max-sm:px-0 h-full max-sm:pb-1 !pt-0 !gap-0"
      >
        <Show when={isLoading()}>
          <div class="flex w-full h-screen justify-center items-center">
            <Loading />
          </div>
        </Show>
        <div class={`contents ${isLoading() ? "hidden" : ""}`}>
          <div class="flex h-fit justify-between items-center px-2 py-1">
            <a
              class="underline text-blue-600 hover:text-blue-800 visited:text-purple-600"
              href={sourceLink()}
            >
              前往源代码
            </a>
            <Show
              when={verificationStatistics()}
              fallback={
                <Button
                  size="xs"
                  onClick={verifyAllOutputsOfOriginalInputs}
                >
                  验证本页全部示例输出
                </Button>
              }
            >
              {(statistics) => (
                <div>
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
                  </Show>。
                </div>
              )}
            </Show>
          </div>
          <div
            class={`${Ankor.WIDGET_OWNER_CLASS} max-h-full h-fit overflow-y-scroll overflow-x-hidden`}
            data-ankor-widget-owner={widgetOwnerData}
          >
            <div class="p-4 tuan-background tuan-prose break-all">
              <div class={`${Ankor.ANCHOR_CLASS} relative z-10`} />
              <div class={Ankor.CONTENT_CLASS} ref={contentContainerEl} />
            </div>
          </div>
        </div>
      </Card>
    </div>
  );
}) satisfies Component;
