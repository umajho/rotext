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

import { Card, Loading } from "../../components/ui/mod";

import "../../styles/tuan-prose";
import {
  syntaxReferenceFilesToHeadingMap,
  syntaxReferenceIndex,
} from "../../data-sources/syntax-reference";
import { getSyntaxReferencePathOfHeading } from "../../utils/syntax-reference";

import { registerCustomElement as registerCustomElementForRotextPreview } from "./RotextPreview/mod";

registerCustomElementForRotextPreview("x-rotext-example");

export default (() => {
  const params = useParams();
  const pageName = createMemo(() => decodeURIComponent(params.pageName!));
  const navigate = useNavigate();
  const location = useLocation();

  let contentContainerEl!: HTMLDivElement;

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

  return (
    <div class="flex max-h-full h-fit justify-center p-8">
      <Card
        class="w-full"
        bodyClass="max-sm:px-1 max-sm:py-1"
      >
        <Show when={isReady()}>
          <div class="flex w-full justify-center">
            <Loading />
          </div>
        </Show>
        <div class="max-h-full h-fit overflow-y-scroll overflow-x-hidden">
          <div
            ref={contentContainerEl}
            class="p-4 tuan-background tuan-prose break-all"
          />
        </div>
      </Card>
    </div>
  );
}) satisfies Component;
