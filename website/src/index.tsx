/* @refresh reload */
import "./index.css";

import { HashRouter, Navigate, Route } from "@solidjs/router";
import { render } from "solid-js/web";

import { Root } from "./components/layout";
import { lazy } from "solid-js";

import NotFoundPage from "./pages/404/mod";

const root = document.getElementById("root");

if (import.meta.env.DEV && !(root instanceof HTMLElement)) {
  throw new Error(
    "Root element not found. Did you forget to add it to your index.html? Or maybe the id attribute got misspelled?",
  );
}

render(() => (
  <HashRouter root={Root}>
    <Route path="/" component={() => <Navigate href={"/playground"} />} />
    <Route
      path="/playground"
      component={lazy(() => import("./pages/Playground/mod"))}
    />

    <Route path="*" component={NotFoundPage} />
  </HashRouter>
), root!);
