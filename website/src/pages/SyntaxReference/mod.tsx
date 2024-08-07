import {
  Component,
  createEffect,
  createMemo,
  createResource,
  createSignal,
  on,
  onCleanup,
  Show,
} from "solid-js";
import { useLocation, useNavigate, useParams } from "@solidjs/router";

import { Button, Card, Loading } from "../../components/ui/mod";

import "../../styles/tuan-prose";
import {
  syntaxReferenceFilesToHeadingMap,
  syntaxReferenceIndex,
} from "../../data-sources/syntax-reference";
import { getSyntaxReferencePathOfHeading } from "../../utils/syntax-reference";

import { registerCustomElement as registerCustomElementForRotextPreview } from "./RotextExample/mod";

// XXX: 一个标签页里只会有一个页面，所以把它作为全局变量也没有问题。
let contentContainerEl!: HTMLDivElement;

function getFixtures(
  fixtureNames: Set<string>,
): { [fixtureName: string]: string } {
  const els = contentContainerEl.querySelectorAll("x-rotext-example-fixture");
  const qualifiedEls = [...els]
    .filter((el) => fixtureNames.has(el.getAttribute("name")!));

  return Object.fromEntries(
    qualifiedEls.map((
      el,
    ) => [el.getAttribute("name")!, el.getAttribute("input")!]),
  );
}

registerCustomElementForRotextPreview("x-rotext-example", { getFixtures });

export default (() => {
  if (contentContainerEl) {
    throw new Error("unreachable");
  }
  onCleanup(() => {
    // @ts-ignore
    contentContainerEl = undefined;
  });

  const params = useParams();
  const pageName = createMemo(() => decodeURIComponent(params.pageName!));
  const navigate = useNavigate();
  const location = useLocation();

  const [isIndexLoaded, setIsIndexLoaded] = createSignal(false);

  const [pageHTMLRaw, { refetch }] = createResource(
    pageName,
    async (pageName) => {
      const map = syntaxReferenceFilesToHeadingMap();
      if (!map) return;
      if (!map[pageName]) {
        navigate("/404");
        return;
      }

      const path = import.meta.env.BASE_URL +
        `static/generated/syntax-reference/${pageName}.inc.html`;
      return (await fetch(path)).text();
    },
  );
  createEffect(on([syntaxReferenceFilesToHeadingMap], () => refetch()));
  createEffect(
    on([pageHTMLRaw, syntaxReferenceIndex], ([pageHTMLRaw, index]) => {
      contentContainerEl.innerHTML = pageHTMLRaw ?? "";

      if (!index) return;
      if (!isIndexLoaded()) {
        setIsIndexLoaded(true);
      }

      const linkEls = contentContainerEl.querySelectorAll("x-internal-link");
      for (const linkEl of linkEls) {
        const heading = linkEl.getAttribute("page-name") ?? linkEl.textContent!;
        const { pathWithAnchor } = getSyntaxReferencePathOfHeading(heading, {
          index,
        });

        const aEl = document.createElement("a");
        aEl.append(...linkEl.childNodes);
        aEl.addEventListener("click", () => navigate(pathWithAnchor));

        linkEl.replaceWith(aEl);
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
  createEffect(on([() => pageHTMLRaw.loading], ([loading]) => {
    if (loading) contentContainerEl.innerHTML = "";
  }));

  const isReady = () => pageHTMLRaw.loading && isIndexLoaded();

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
  createEffect(on([pageHTMLRaw], () => setVerificationStatistics(null)));
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

  return (
    <div class="flex max-h-full h-fit justify-center p-2 sm:p-4 lg:p-6 2xl:p-8">
      <Card
        class="w-full"
        bodyClass="max-sm:px-1 !pt-0"
      >
        <div class="h-full">
          <Show when={isReady()}>
            <div class="flex w-full justify-center">
              <Loading />
            </div>
          </Show>
          <div class="flex h-fit items-center px-2">
            <div class="flex-1" />
            <div>
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
                  <>
                    本页示例输出验证结果：匹配{" "}
                    <span class="text-green-500">{statistics().matches}
                    </span>，不匹配{" "}
                    <span class="text-red-500">{statistics().mismatches}</span>
                    <Show when={verificationStatisticsUnverified()}>
                      {(unverified) => (
                        <>
                          ，尚未验证{" "}
                          <span class="text-gray-500">{unverified()}</span>
                        </>
                      )}
                    </Show>。
                  </>
                )}
              </Show>
            </div>
          </div>
          <div class="max-h-full h-fit overflow-y-scroll overflow-x-hidden">
            <div
              ref={contentContainerEl}
              class="p-4 tuan-background tuan-prose break-all"
            />
          </div>
        </div>
      </Card>
    </div>
  );
}) satisfies Component;
