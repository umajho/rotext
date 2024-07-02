/* @refresh reload */
import "./index.css";

import { HashRouter, Route } from "@solidjs/router";
import { render } from "solid-js/web";

import { Root } from "./components/layout";
import { lazy } from "solid-js";

const root = document.getElementById("root");

if (import.meta.env.DEV && !(root instanceof HTMLElement)) {
  throw new Error(
    "Root element not found. Did you forget to add it to your index.html? Or maybe the id attribute got misspelled?",
  );
}

render(() => (
  <HashRouter root={Root}>
    <Route path="/" component={lazy(() => import("./App"))} />
  </HashRouter>
), root!);
