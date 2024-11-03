import { lazy } from "solid-js";
import { HashRouter, Navigate, Route } from "@solidjs/router";
import { render } from "solid-js/web";

import { RotextProcessorName } from "./hooks/rotext-processors-store";
import { RotextProcessorsStoreProvider } from "./contexts/rotext-processors-store";
import { registerCustomElementsOnce } from "./custom-elements/mod";

import { Root } from "./components/layout";

import NotFoundPage from "./pages/404/mod";

const CURRENT_ROTEXT_PROCESSOR_NAME_LOCAL_KEY = "currentRotextProcessorName";

const root = document.getElementById("root");

if (import.meta.env.DEV && !(root instanceof HTMLElement)) {
  throw new Error(
    "Root element not found. Did you forget to add it to your index.html? Or maybe the id attribute got misspelled?",
  );
}

registerCustomElementsOnce();

render(() => {
  return (
    <RotextProcessorsStoreProvider
      initialProcessorName={getCurrentRotextProcessorNameInLocalStorage()}
      onCurrentProcessorNameChange={setCurrentRotextProcessorNameInLocalStorage}
    >
      <HashRouter root={Root} explicitLinks={true}>
        <Route path="/" component={() => <Navigate href={"/playground"} />} />
        <Route
          path="/playground"
          component={lazy(() => import("./pages/Playground/mod"))}
        />
        <Route
          path="/wiki/*pageName"
          component={lazy(() => import("./pages/Wiki/mod"))}
        />

        <Route path="*" component={NotFoundPage} />
      </HashRouter>
    </RotextProcessorsStoreProvider>
  );
}, root!);

function getCurrentRotextProcessorNameInLocalStorage(): RotextProcessorName {
  const item = localStorage.getItem(
    CURRENT_ROTEXT_PROCESSOR_NAME_LOCAL_KEY,
  );
  if (item === "rust") return item;
  return "rust";
}
function setCurrentRotextProcessorNameInLocalStorage(
  name: RotextProcessorName,
) {
  localStorage.setItem(CURRENT_ROTEXT_PROCESSOR_NAME_LOCAL_KEY, name);
}
